//! The usb module contains a wrapper struct to more conveniently manage a usb device and hid class.

use core::ops::{Deref, DerefMut};

use keyberon::{keyboard::Leds, new_class, new_device, Class};
use usb_device::{class_prelude::*, prelude::*};

/// A wrapper around the usb device and hid class for convenience.
/// Most behavior is deferred to the hid class.
pub struct Usb<'a, B: UsbBus, L: Leds> {
    /// This is the usb device managing IO and polling for updates.
    dev: UsbDevice<'a, B>,
    /// This is the keyboard usb hid class, it handles reporting keyboard state through the device.
    hid: Class<'a, B, L>,
}

impl<'a, B: UsbBus, L: Leds> Usb<'a, B, L> {
    pub fn new(bus: &'a UsbBusAllocator<B>, leds: L) -> Self {
        Self {
            dev: new_device(bus),
            hid: new_class(bus, leds),
        }
    }

    /// Wraps polling the usb device and hid together.
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
