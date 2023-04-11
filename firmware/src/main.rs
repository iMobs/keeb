#![no_std]
#![no_main]

use core::ops::{Deref, DerefMut};

use defmt_rtt as _;
use panic_probe as _;
use stm32f1xx_hal as _;
use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::hid_class::HIDClass;

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    use stm32f1xx_hal::{
        pac::TIM3,
        prelude::*,
        timer::{CounterHz, Event},
        usb::{Peripheral, UsbBus, UsbBusType},
        watchdog::IndependentWatchdog,
    };
    use usb_device::{class_prelude::*, prelude::*};
    use usbd_hid::hid_class::{
        HIDClass, HidClassSettings, HidCountryCode, HidProtocol, HidSubClass, ProtocolModeConfig,
    };

    use crate::Usb;

    #[local]
    struct Local {
        timer: CounterHz<TIM3>,
        watchdog: IndependentWatchdog,
    }

    #[shared]
    struct Shared {
        usb: Usb<'static, UsbBusType>,
    }

    #[init(local = [usb_bus: Option<UsbBusAllocator<UsbBusType>> = None])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
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

        let gpioa = device.GPIOA.split();

        let usb = Peripheral {
            usb: device.USB,
            pin_dm: gpioa.pa11,
            pin_dp: gpioa.pa12,
        };

        let usb_bus = cx.local.usb_bus.insert(UsbBus::new(usb));

        // https://github.com/obdev/v-usb/blob/7a28fdc685952412dad2b8842429127bc1cf9fa7/usbdrv/USB-IDs-for-free.txt#L128
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27db))
            .manufacturer("Mobs Workshop")
            .product("Keeb")
            .build();

        let usb_hid = HIDClass::new_with_settings(
            usb_bus,
            &[0], // TODO: add descriptor
            1,
            HidClassSettings {
                subclass: HidSubClass::NoSubClass,
                protocol: HidProtocol::Keyboard,
                config: ProtocolModeConfig::ForceReport,
                locale: HidCountryCode::US,
            },
        );

        let mut timer = device.TIM3.counter_hz(&clocks);
        timer.listen(Event::Update);
        timer.start(1.kHz()).unwrap();

        // timer is running 10x faster so should catch a problem fast
        let mut watchdog = IndependentWatchdog::new(device.IWDG);
        watchdog.start(10.millis());

        (
            Shared {
                usb: Usb::new(usb_dev, usb_hid),
            },
            Local { timer, watchdog },
            init::Monotonics(),
        )
    }

    #[task(binds = TIM3, priority = 1, local = [timer, watchdog], shared = [usb])]
    fn tick(cx: tick::Context) {
        defmt::info!("tick");

        cx.local.timer.clear_interrupt(Event::Update);
        cx.local.watchdog.feed();

        // TODO: scan keyboard
    }

    #[task(binds = USB_HP_CAN_TX, priority = 2, shared = [usb])]
    fn usb_tx(mut cx: usb_tx::Context) {
        defmt::info!("usb_tx");

        cx.shared.usb.lock(|usb| usb.poll());
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 2, shared = [usb])]
    fn usb_rx(mut cx: usb_rx::Context) {
        defmt::info!("usb_rx");

        cx.shared.usb.lock(|usb| usb.poll());
    }
}

pub struct Usb<'a, B: UsbBus> {
    dev: UsbDevice<'a, B>,
    hid: HIDClass<'a, B>,
}

impl<'a, B: UsbBus> Usb<'a, B> {
    fn new(dev: UsbDevice<'a, B>, hid: HIDClass<'a, B>) -> Self {
        Self { dev, hid }
    }

    fn poll(&mut self) {
        if self.dev.poll(&mut [&mut self.hid]) {
            // FIXME: since this is a stub I don't think it's necessary
            self.hid.poll();
        }
    }
}

impl<'a, B: UsbBus> Deref for Usb<'a, B> {
    type Target = HIDClass<'a, B>;

    fn deref(&self) -> &Self::Target {
        &self.hid
    }
}

impl<'a, B: UsbBus> DerefMut for Usb<'a, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hid
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
