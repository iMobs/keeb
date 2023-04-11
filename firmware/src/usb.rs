use core::ops::{Deref, DerefMut};

use keyberon::{keyboard::Leds, Class};
use usb_device::{class_prelude::*, prelude::*};

pub struct Usb<'a, B: UsbBus, L: Leds> {
    dev: UsbDevice<'a, B>,
    hid: Class<'a, B, L>,
}

impl<'a, B: UsbBus, L: Leds> Usb<'a, B, L> {
    pub fn new(dev: UsbDevice<'a, B>, hid: Class<'a, B, L>) -> Self {
        Self { dev, hid }
    }

    pub fn poll(&mut self) -> bool {
        self.dev.poll(&mut [&mut self.hid])
    }
}

impl<'a, B: UsbBus, L: Leds> Deref for Usb<'a, B, L> {
    type Target = Class<'a, B, L>;

    fn deref(&self) -> &Self::Target {
        &self.hid
    }
}

impl<'a, B: UsbBus, L: Leds> DerefMut for Usb<'a, B, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hid
    }
}
