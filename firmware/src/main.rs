//! The Keeb firmware, using [RTIC](https://rtic.rs) and [keyberon](https://github.com/TeXitoi/keyberon).

#![no_std]
#![no_main]

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    use keeb::{
        configure_clocks,
        layout::{LAYERS, NUM_COLS, NUM_LAYERS, NUM_ROWS},
        leds::Leds,
        pins::{pins, ErasedCol, ErasedRow},
        usb::Usb,
    };
    use keyberon::{debounce::Debouncer, layout::Layout, matrix::Matrix};
    use stm32f1xx_hal::{
        pac::TIM3,
        prelude::*,
        timer::{CounterHz, Event},
        usb::{Peripheral, UsbBus, UsbBusType},
        // watchdog::IndependentWatchdog,
    };
    use usb_device::bus::UsbBusAllocator;

    #[local]
    struct Local {
        layout: Layout<NUM_COLS, NUM_ROWS, NUM_LAYERS>,
        matrix: Matrix<ErasedCol, ErasedRow, NUM_COLS, NUM_ROWS>,
        timer: CounterHz<TIM3>,
        // watchdog: IndependentWatchdog,
    }

    #[shared]
    struct Shared {
        usb: Usb<'static, UsbBusType, Leds>,
    }

    #[init(local = [usb_bus: Option<UsbBusAllocator<UsbBusType>> = None])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::trace!("init");

        let device = cx.device;

        let clocks = configure_clocks(device.FLASH, device.RCC);

        let layout = Layout::new(&LAYERS);

        let pins = pins(device.GPIOA, device.GPIOB);

        macro_rules! ep {
            ($p: expr) => ($p.erase());
            ($($p: expr),+ $(,)?) => (
                [$(ep!($p)),+]
            );
        }

        let matrix = Matrix::new(
            ep![
                pins.col0, pins.col1, pins.col2, pins.col3, pins.col4, pins.col5, pins.col6,
                pins.col7, pins.col8, pins.col9, pins.col10, pins.col11, pins.col12, pins.col13,
            ],
            ep![pins.row0, pins.row1, pins.row2, pins.row3, pins.row4, pins.row5,],
        )
        .unwrap();

        let usb = Peripheral {
            usb: device.USB,
            pin_dm: pins.usb_dm,
            pin_dp: pins.usb_dp,
        };

        let usb_bus = cx.local.usb_bus.insert(UsbBus::new(usb));
        let leds = Leds::new(pins.caps_lock.erase());
        let usb = Usb::new(usb_bus, leds);

        let mut timer = device.TIM3.counter_hz(&clocks);
        timer.listen(Event::Update);
        timer.start(1.kHz()).unwrap();

        // timer is running 10x faster so should catch a problem fast
        // let mut watchdog = IndependentWatchdog::new(device.IWDG);
        // watchdog.start(10.millis());

        (
            Shared { usb },
            Local {
                layout,
                matrix,
                timer,
                // watchdog,
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
            // watchdog,
        ],
        shared = [usb],
    )]
    fn tick(cx: tick::Context) {
        let tick::LocalResources {
            debouncer,
            layout,
            matrix,
            timer,
            // watchdog,
        } = cx.local;
        let tick::SharedResources { mut usb } = cx.shared;

        timer.clear_interrupt(Event::Update);
        // watchdog.feed();

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
