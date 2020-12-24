use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Keyboard {
    keys: [u8; 16],
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard { keys: [0; 16] }
    }

    pub fn reset_keys(&mut self) {
        self.keys = [0; 16]
    }

    pub fn get_internal_array(&self) -> *const u8 {
        self.keys.as_ptr()
    }

    pub fn set_key(&mut self, key: u8) {
        self.keys[key as usize] = 1;
    }

    pub fn key_is_pressed(&self, key: u8) -> bool {
        self.keys[key as usize] != 0
    }
}

#[cfg(test)]
mod keyboard_tests {
    use super::Keyboard;
}
