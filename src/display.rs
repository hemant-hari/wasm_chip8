use std::fmt;
use wasm_bindgen::prelude::*;

use super::utils;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pixel {
    Off = 0,
    On = 1,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Display {
    width: u32,
    height: u32,
    pixels: Vec<Pixel>,
}

#[wasm_bindgen]
impl Display {
    pub fn new_default() -> Display {
        utils::set_panic_hook();
        let width = 64;
        let height = 64;

        let pixels = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Pixel::On
                } else {
                    Pixel::Off
                }
            })
            .collect();

        Display::new(width, height, Some(pixels))
    }

    pub fn new_empty() -> Display {
        utils::set_panic_hook();
        let width = 64;
        let height = 32;
        let pixels = (0..width * height).map(|_| Pixel::Off).collect();

        Display::new(width, height, Some(pixels))
    }

    fn new(width: u32, height: u32, pixels: Option<Vec<Pixel>>) -> Display {
        Display {
            width,
            height,
            pixels: pixels.unwrap_or(Vec::with_capacity((width * height) as usize)),
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> *const Pixel {
        self.pixels.as_ptr()
    }

    pub fn cls(&mut self) {
        self.pixels = (0..self.width * self.height).map(|_| Pixel::Off).collect();
    }

    pub fn toggle_pixel(&mut self, i: usize) {
        self.pixels[i] = if self.pixels[i] == Pixel::Off {
            Pixel::On
        } else {
            Pixel::Off
        };
    }

    pub fn get_pixel(&self, i: usize) -> Pixel {
        self.pixels[i]
    }

    pub fn draw_bytes(&mut self, x: u8, y: u8, bytes: &[u8]) -> bool {
        let bits: Vec<[bool; 8]> = bytes.into_iter().map(Display::to_bool_array).collect();
        let mut collision_flag = false;

        for (i_y, pos_y) in (y..(y + bytes.len() as u8)).enumerate() {
            for (i_x, pos_x) in (x..(x + 8)).enumerate() {
                let idx = self.get_index(pos_y as u32 % self.height, pos_x as u32 % self.width);
                self.pixels[idx] = match (self.pixels[idx], bits[i_y][i_x]) {
                    (Pixel::On, true) => {
                        collision_flag = true;
                        Pixel::Off
                    }
                    (Pixel::Off, false) => Pixel::Off,
                    (Pixel::On, false) => Pixel::On,
                    (Pixel::Off, true) => Pixel::On,
                };
            }
        }
        collision_flag
    }

    fn to_bool_array(bits: &u8) -> [bool; 8] {
        let mut bool_array: [bool; 8] = [false; 8];
        for i in 0..8 {
            bool_array[i] = ((bits >> (7 - i)) & 1) != 0;
        }

        bool_array
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.pixels[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        let mut next = self.pixels.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.pixels[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Pixel::On, x) if x < 2 => Pixel::Off,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Pixel::On, 2) | (Pixel::On, 3) => Pixel::On,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Pixel::On, x) if x > 3 => Pixel::Off,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Pixel::Off, 3) => Pixel::On,
                    // All other pixels remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.pixels = next;
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.pixels.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Pixel::Off { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

pub static FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn it_gets_zero_index() {
        let test_disp = Display::new(6, 6, None);
        assert_eq!(test_disp.get_index(0, 0), 0);
    }

    #[test]
    fn it_gets_column() {
        let test_disp = Display::new(6, 6, None);
        assert_eq!(test_disp.get_index(0, 5), 5)
    }
}
