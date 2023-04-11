use keyberon::keyboard;
use stm32f1xx_hal::gpio::{ErasedPin, Output};

/// Leds is used to manage output pins for keyboard status.
///
/// Fun fact, the host OS tells the keyboard if caps lock is on and not the other way around.
pub struct Leds {
    /// The output pin for controlling the caps lock LED.
    caps_lock: ErasedPin<Output>,
}

impl Leds {
    pub fn new(caps_lock: ErasedPin<Output>) -> Self {
        Self { caps_lock }
    }
}

impl keyboard::Leds for Leds {
    fn caps_lock(&mut self, status: bool) {
        if status {
            self.caps_lock.set_high();
        } else {
            self.caps_lock.set_low();
        };
    }
}
