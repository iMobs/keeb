#![no_main]
#![no_std]

pub mod layout;
pub mod leds;
pub mod pins;
pub mod usb;

use defmt_rtt as _;
use panic_probe as _;
use stm32f1xx_hal::{
    pac::{FLASH, RCC},
    prelude::*,
    rcc::Clocks,
};

/// Standard initialization of clocks at 48 MHz
///
/// TBD if an external crystal will be necessary and what speed.
pub fn configure_clocks(flash: FLASH, rcc: RCC) -> Clocks {
    let mut flash = flash.constrain();
    let rcc = rcc.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(12.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    debug_assert!(clocks.usbclk_valid());

    clocks
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
