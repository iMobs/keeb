use keyberon::keyboard;

// TODO: this should control some output pins
pub struct Leds;

impl Leds {
    pub fn win_lock(&mut self, _status: bool) {}
}

impl keyboard::Leds for Leds {
    fn caps_lock(&mut self, _status: bool) {
        // TODO
    }
}
