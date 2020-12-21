pub struct Keyboard {
    keys: u16,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard { keys: 0 }
    }

    pub fn set_keys(&mut self, key_bytes: u16) {
        self.keys = key_bytes;
    }

    pub fn set_key(&mut self, key: u8) {
        self.keys |= 1 << key;
    }

    pub fn unset_key(&mut self, key: u8) {
        self.keys &= !(1 << key);
    }

    pub fn toggle_key(&mut self, key: u8) {
        self.keys ^= 1 << key;
    }

    pub fn key_pressed(&self, key: u8) -> bool {
        self.keys >> key & 1 != 0
    }
}

#[cfg(test)]
mod keyboard_tests {
    use super::Keyboard;

    #[test]
    fn it_gets_key_one() {
        let mut kb = Keyboard::new();
        kb.set_keys(0b0000_0001);
        assert!(kb.key_pressed(0x0));
        assert!(!kb.key_pressed(0x1));
    }

    #[test]
    fn it_gets_key_f() {
        let mut kb = Keyboard::new();
        kb.set_keys(0b1000_0000_0000_0000);
        assert!(kb.key_pressed(0xF));
        assert!(!kb.key_pressed(0x1));
    }

    #[test]
    fn it_sets_key_f() {
        let mut kb = Keyboard::new();
        kb.set_key(0xF);
        assert!(kb.key_pressed(0xF));
        assert!(!kb.key_pressed(0x1));
    }

    #[test]
    fn it_unsets_key_f() {
        let mut kb = Keyboard::new();
        kb.set_key(0xF);
        assert!(kb.key_pressed(0xF));
        kb.unset_key(0xF);
        assert!(!kb.key_pressed(0xF));
    }

    #[test]
    fn it_toggles_key_9() {
        let mut kb = Keyboard::new();
        kb.set_key(0x9);
        assert!(kb.key_pressed(0x9));
        kb.toggle_key(0x9);
        assert!(!kb.key_pressed(0x9));
        kb.toggle_key(0x9);
        assert!(kb.key_pressed(0x9));
    }
}
