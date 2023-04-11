//! The Keeb firmware, using [RTIC](https://rtic.rs) and [keyberon](https://github.com/TeXitoi/keyberon).

#![no_std]
#![no_main]

mod layout;
mod leds;
mod usb;

use defmt_rtt as _;
use keyberon::{debounce::Debouncer, layout::Layout, matrix::Matrix};
use panic_probe as _;
use stm32f1xx_hal::{
    gpio::{ErasedPin, Input, Output, PullUp},
    pac::TIM3,
    prelude::*,
    timer::{CounterHz, Event},
    usb::{Peripheral, UsbBus, UsbBusType},
    watchdog::IndependentWatchdog,
};
use usb::Usb;
use usb_device::bus::UsbBusAllocator;

const NUM_COLS: usize = 14;
const NUM_ROWS: usize = 6;
const NUM_LAYERS: usize = 2;

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    use super::*;

    #[local]
    struct Local {
        layout: Layout<NUM_COLS, NUM_ROWS, NUM_LAYERS>,
        matrix: Matrix<ErasedPin<Input<PullUp>>, ErasedPin<Output>, NUM_COLS, NUM_ROWS>,
        timer: CounterHz<TIM3>,
        watchdog: IndependentWatchdog,
    }

    #[shared]
    struct Shared {
        usb: Usb<'static, UsbBusType, ()>,
    }

    #[init(local = [usb_bus: Option<UsbBusAllocator<UsbBusType>> = None])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::trace!("init");

        let device = cx.device;

        let mut flash = device.FLASH.constrain();
        let rcc = device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(12.MHz())
            .sysclk(48.MHz())
            .pclk1(24.MHz())
            .freeze(&mut flash.acr);

        debug_assert!(clocks.usbclk_valid());

        let layout = Layout::new(&layout::LAYERS);

        let mut gpioa = device.GPIOA.split();
        let mut gpiob = device.GPIOB.split();

        // TODO: correct these pin assignments once design is done
        let matrix = Matrix::new(
            [
                gpiob.pb0.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb1.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb2.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb5.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb6.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb7.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb8.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb9.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb10.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb11.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb12.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb13.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb14.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb15.into_pull_up_input(&mut gpiob.crh).erase(),
            ],
            [
                gpioa.pa0.into_push_pull_output(&mut gpioa.crl).erase(),
                gpioa.pa1.into_push_pull_output(&mut gpioa.crl).erase(),
                gpioa.pa2.into_push_pull_output(&mut gpioa.crl).erase(),
                gpioa.pa3.into_push_pull_output(&mut gpioa.crl).erase(),
                gpioa.pa4.into_push_pull_output(&mut gpioa.crl).erase(),
                gpioa.pa5.into_push_pull_output(&mut gpioa.crl).erase(),
            ],
        )
        .unwrap();

        let usb = Peripheral {
            usb: device.USB,
            pin_dm: gpioa.pa11,
            pin_dp: gpioa.pa12,
        };

        let usb_bus = cx.local.usb_bus.insert(UsbBus::new(usb));
        // TODO: setup leds
        let usb = Usb::new(usb_bus, ());

        let mut timer = device.TIM3.counter_hz(&clocks);
        timer.listen(Event::Update);
        timer.start(1.kHz()).unwrap();

        // timer is running 10x faster so should catch a problem fast
        let mut watchdog = IndependentWatchdog::new(device.IWDG);
        watchdog.start(10.millis());

        (
            Shared { usb },
            Local {
                layout,
                matrix,
                timer,
                watchdog,
            },
            init::Monotonics(),
        )
    }

    #[task(
        binds = TIM3,
        priority = 1,
        local = [
            debouncer: Debouncer<[[bool; NUM_COLS]; NUM_ROWS]> = Debouncer::new(
                [[false; NUM_COLS]; NUM_ROWS],
                [[false; NUM_COLS]; NUM_ROWS],
                5,
            ),
            layout,
            matrix,
            timer,
            watchdog,
        ],
        shared = [usb],
    )]
    fn tick(cx: tick::Context) {
        let tick::LocalResources {
            debouncer,
            layout,
            matrix,
            timer,
            watchdog,
        } = cx.local;
        let tick::SharedResources { mut usb } = cx.shared;

        timer.clear_interrupt(Event::Update);
        watchdog.feed();

        for event in debouncer.events(matrix.get().unwrap()) {
            layout.event(event);
        }

        // If there are ever custom events handle them here.
        layout.tick();

        let report = layout.keycodes().collect();

        // I think the report is picked up within usb_device when polled
        usb.lock(|usb| usb.device_mut().set_keyboard_report(report));
    }

    #[task(binds = USB_HP_CAN_TX, priority = 2, shared = [usb])]
    fn usb_tx(mut cx: usb_tx::Context) {
        defmt::trace!("usb_tx");
        cx.shared.usb.lock(|usb| usb.poll());
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 2, shared = [usb])]
    fn usb_rx(mut cx: usb_rx::Context) {
        defmt::trace!("usb_rx");
        cx.shared.usb.lock(|usb| usb.poll());
    }
}

defmt::timestamp!("{=u32:us}", {
    use core::sync::atomic::{AtomicU32, Ordering};
    static COUNT: AtomicU32 = AtomicU32::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed)
});

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
