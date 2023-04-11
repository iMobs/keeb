#![no_std]
#![no_main]

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
    };
    use usb_device::{class_prelude::*, prelude::*};
    use usbd_hid::hid_class::{
        HIDClass, HidClassSettings, HidCountryCode, HidProtocol, HidSubClass, ProtocolModeConfig,
    };

    #[local]
    struct Local {
        timer: CounterHz<TIM3>,
    }

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBusType>,
        usb_hid: HIDClass<'static, UsbBusType>,
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
            .supports_remote_wakeup(true)
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

        (
            Shared { usb_dev, usb_hid },
            Local { timer },
            init::Monotonics(),
        )
    }

    #[task(binds = TIM3, local = [timer])]
    fn tick(cx: tick::Context) {
        defmt::info!("tick");

        cx.local.timer.clear_interrupt(Event::Update);
    }

    #[task(binds = USB_HP_CAN_TX, shared = [usb_dev, usb_hid])]
    fn usb_tx(cx: usb_tx::Context) {
        defmt::info!("usb_tx");

        let usb_tx::SharedResources { usb_dev, usb_hid } = cx.shared;

        (usb_dev, usb_hid).lock(|usb_dev, usb_hid| super::usb_poll(usb_dev, usb_hid));
    }

    #[task(binds = USB_LP_CAN_RX0, shared = [usb_dev, usb_hid])]
    fn usb_rx(cx: usb_rx::Context) {
        defmt::info!("usb_rx");

        let usb_rx::SharedResources { usb_dev, usb_hid } = cx.shared;

        (usb_dev, usb_hid).lock(|usb_dev, usb_hid| super::usb_poll(usb_dev, usb_hid));
    }
}

fn usb_poll<B: UsbBus>(usb_dev: &mut UsbDevice<B>, usb_hid: &mut HIDClass<B>) {
    if usb_dev.poll(&mut [usb_hid]) {
        // TODO: send report?
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
