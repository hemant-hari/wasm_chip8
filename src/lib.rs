mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod display;
pub mod keyboard;

pub mod chip8 {
    use crate::display::{self, Display};
    use crate::keyboard::Keyboard;

    pub struct Cpu {
        i: u16,
        pc: u16,
        s_ptr: u8,
        stack: [u16; 16],
        delay: u8,
        registers: [u8; 16],
        memory: [u8; 4096],
        display: Display,
        keyboard: Keyboard,
    }

    impl Cpu {
        pub fn new() -> Cpu {
            Cpu {
                i: 0,
                pc: 0,
                s_ptr: 0,
                stack: [0; 16],
                delay: 0,
                registers: [0; 16],
                memory: [0; 4096],
                display: Display::new_empty(),
                keyboard: Keyboard::new(),
            }
        }

        pub fn memory(&'static self) -> &'static [u8] {
            &self.memory
        }

        fn run_opcode(&mut self, opcode: u16) {
            let nibbles: [u16; 4] = Cpu::get_nibbles(opcode);
            let nnn = || opcode & 0x0FFF;
            let kk = || (opcode & 0x00FF) as u8;
            let x = || ((opcode & 0x0F00) >> 8) as usize;
            let y = || ((opcode & 0x00F0) >> 4) as usize;

            match nibbles {
                [0, 0, 0xE, 0] => self.display.cls(),
                [0, 0, 0xE, 0xE] => self.ret_subroutine(),
                [1, _, _, _] => self.pc = nnn(),
                [2, _, _, _] => self.call_subroutine(nnn()),
                [3, _, _, _] if self.registers[x()] == kk() => self.pc += 2,
                [4, _, _, _] if self.registers[x()] != kk() => self.pc += 2,
                [5, _, _, 0] if self.registers[x()] == self.registers[y()] => self.pc += 2,
                [6, _, _, _] => self.registers[x()] = kk(),
                [7, _, _, _] => self.registers[x()] += kk(),
                [8, _, _, 0] => self.registers[x()] = self.registers[y()],
                [8, _, _, 1] => self.registers[x()] |= self.registers[y()],
                [8, _, _, 2] => self.registers[x()] &= self.registers[y()],
                [8, _, _, 3] => self.registers[x()] ^= self.registers[y()],
                [8, _, _, 4] => self.registers[x()] = self.safe_add_registers(x(), y()),
                [8, _, _, 5] => self.registers[x()] = self.safe_sub_registers(x(), y()),
                [8, _, _, 6] => self.registers[x()] = self.halve(x()),
                [8, _, _, 7] => self.registers[x()] = self.safe_sub_registers(y(), x()),
                [8, _, _, 0xE] => self.registers[x()] = self.double(x()),
                [9, _, _, 0] if self.registers[x()] != self.registers[y()] => self.pc += 2,
                [0xA, _, _, _] => self.i = nnn(),
                [0xB, _, _, _] => self.pc = nnn() + self.registers[0] as u16,
                _ => return,
            }
        }

        fn get_nibbles(opcode: u16) -> [u16; 4] {
            [
                (opcode & 0xF000) >> 12,
                (opcode & 0x0F00) >> 8,
                (opcode & 0x00F0) >> 4,
                (opcode & 0x000F),
            ]
        }

        fn call_subroutine(&mut self, nnn: u16) {
            self.s_ptr += 1;
            self.stack[self.s_ptr as usize] = self.pc;
            self.pc = nnn;
        }

        fn ret_subroutine(&mut self) {
            self.pc = self.stack[self.s_ptr as usize];
            self.s_ptr -= 1;
        }

        fn safe_add_registers(&mut self, x: usize, y: usize) -> u8 {
            let sum: u16 = self.registers[x] as u16 + self.registers[y] as u16;
            if sum > 0xFF {
                self.registers[0xF] = 1
            }
            sum as u8
        }

        fn safe_sub_registers(&mut self, first: usize, second: usize) -> u8 {
            let reg_first = self.registers[first];
            let reg_second = self.registers[second];
            if reg_first > reg_second {
                self.registers[0xF] = 1;
                reg_first - reg_second
            } else {
                self.registers[0xF] = 0;
                (0x100 + reg_first as u16 - reg_second as u16) as u8
            }
        }

        fn halve(&mut self, x: usize) -> u8 {
            self.registers[0xF] = self.registers[x] & 1;
            self.registers[x] >> 1
        }

        fn double(&mut self, x: usize) -> u8 {
            self.registers[0xF] = self.registers[x] & 0b1000_0000 >> 7;
            self.registers[x] << 1
        }
    }

    #[cfg(test)]
    mod cpu_tests {
        use super::*;
        use crate::display::Cell;

        //00E0
        #[test]
        fn it_clears_screen() {
            let mut cpu = Cpu::new();
            cpu.display.toggle_cell(1);
            cpu.run_opcode(0x00E0);
            assert!(cpu.display.get_cell(1) == Cell::Dead);
        }

        #[test]
        fn it_splits_nibbles() {
            let nibbles = Cpu::get_nibbles(0x1234);
            let expected_nibbles: [u16; 4] = [1, 2, 3, 4];
            assert_eq!(expected_nibbles, nibbles);
        }

        //1nnn
        #[test]
        fn it_sets_pc() {
            let mut cpu = Cpu::new();
            cpu.run_opcode(0x1123);
            assert_eq!(cpu.pc, 0x123);
        }

        //2nnn
        #[test]
        fn it_calls_subroutine() {
            let mut cpu = Cpu::new();
            cpu.run_opcode(0x2123);
            assert_eq!(cpu.s_ptr, 1);
            assert_eq!(cpu.stack[0], 0);
            assert_eq!(cpu.pc, 0x123);
        }

        //00EE
        #[test]
        fn it_returns_from_subroutine() {
            let mut cpu = Cpu::new();
            cpu.s_ptr = 1;
            cpu.stack[1] = 0x321;
            cpu.pc = 0x123;
            cpu.run_opcode(0x00EE);
            assert_eq!(cpu.pc, 0x321);
        }

        //3xkk
        #[test]
        fn it_skips_if_equal() {
            let mut cpu = Cpu::new();
            cpu.registers[0xF] = 0xAA;
            cpu.pc = 10;
            cpu.run_opcode(0x3FAA);
            assert_eq!(12, cpu.pc);
        }

        //3xkk
        #[test]
        fn it_doesnt_skip_if_not_equal() {
            let mut cpu = Cpu::new();
            cpu.registers[0xF] = 0xA1;
            cpu.pc = 10;
            cpu.run_opcode(0x3FAA);
            assert_eq!(10, cpu.pc);
        }

        //4xkk
        #[test]
        fn it_skips_if_not_equal() {
            let mut cpu = Cpu::new();
            cpu.registers[0xF] = 0xA1;
            cpu.pc = 10;
            cpu.run_opcode(0x4FAA);
            assert_eq!(12, cpu.pc);
        }

        //4xkk
        #[test]
        fn it_doesnt_skip_if_equal() {
            let mut cpu = Cpu::new();
            cpu.registers[0xF] = 0xA1;
            cpu.pc = 10;
            cpu.run_opcode(0x4FA1);
            assert_eq!(10, cpu.pc);
        }

        //5xy0
        #[test]
        fn it_skips_if_registers_equal() {
            let mut cpu = Cpu::new();
            cpu.registers[0xF] = 0xAA;
            cpu.registers[0xD] = 0xAA;
            cpu.pc = 10;
            cpu.run_opcode(0x5FD0);
            assert_eq!(12, cpu.pc);
        }

        //6xkk
        #[test]
        fn it_sets_reg_value() {
            let mut cpu = Cpu::new();
            cpu.run_opcode(0x6BAB);
            assert_eq!(cpu.registers[0xB], 0xAB);
        }

        //7xkk
        #[test]
        fn it_adds_and_sets_reg_value() {
            let mut cpu = Cpu::new();
            cpu.registers[0xB] = 0x0A;
            cpu.run_opcode(0x7BAB);
            assert_eq!(cpu.registers[0xB], 0xAB + 0x0A);
        }

        //8xy0
        #[test]
        fn it_copies_register_values_to_x_from_y() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x1C; //Vx
            cpu.registers[0x5] = 0x10; //Vy
            cpu.run_opcode(0x8150);
            assert_eq!(cpu.registers[0x1], 0x10);
        }

        //8xy1
        #[test]
        fn it_bitwise_or_register_values_and_store_to_x() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x1C; //Vx
            cpu.registers[0x5] = 0x10; //Vy
            cpu.run_opcode(0x8151);
            assert_eq!(cpu.registers[0x1], 0x10 | 0x1C);
        }

        //8xy2
        #[test]
        fn it_bitwise_and_register_values_and_store_to_x() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x1C; //Vx
            cpu.registers[0x5] = 0x10; //Vy
            cpu.run_opcode(0x8152);
            assert_eq!(cpu.registers[0x1], 0x10 & 0x1C);
        }

        //8xy3
        #[test]
        fn it_bitwise_xor_register_values_and_store_to_x() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x1C; //Vx
            cpu.registers[0x5] = 0x10; //Vy
            cpu.run_opcode(0x8153);
            assert_eq!(cpu.registers[0x1], 0x10 ^ 0x1C);
        }

        //8xy4
        #[test]
        fn it_adds_register_values_and_store_to_x() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x1C; //Vx
            cpu.registers[0x5] = 0x10; //Vy
            cpu.run_opcode(0x8154);
            assert_eq!(cpu.registers[0x1], 0x10 + 0x1C);
        }

        //8xy4
        #[test]
        fn it_overflows_on_add_register_values_and_store_to_x() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0xFF; //Vx
            cpu.registers[0x5] = 0xFF; //Vy
            cpu.run_opcode(0x8154);
            assert_eq!(cpu.registers[0x1], ((0xFF + 0xFF) & 0xFF) as u8);
            assert_eq!(cpu.registers[0xF], 1);
        }

        //8xy5
        #[test]
        fn it_subtracts_register_values_and_store_to_x() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x1C; //Vx
            cpu.registers[0x5] = 0x10; //Vy
            cpu.run_opcode(0x8155);
            assert_eq!(cpu.registers[0x1], 0x1C - 0x10);
            assert_eq!(cpu.registers[0xF], 1);
        }

        //8xy5
        #[test]
        fn it_carries_subtracts_register_values_and_store_to_x() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x10; //Vx
            cpu.registers[0x5] = 0x1C; //Vy
            cpu.run_opcode(0x8155);
            assert_eq!(cpu.registers[0x1], (0x110 - 0x1C) as u8);
            assert_eq!(cpu.registers[0xF], 0);
        }

        //8xy6
        #[test]
        fn it_sets_vf_if_lsb_1() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x11; //Vx
            cpu.run_opcode(0x8156);
            assert_eq!(cpu.registers[0xF], 1);
            assert_eq!(cpu.registers[0x1], 0x10 / 2);
        }

        //8xy6
        #[test]
        fn it_unsets_vf_if_lsb_0() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x10; //Vx
            cpu.run_opcode(0x8156);
            assert_eq!(cpu.registers[0xF], 0);
            assert_eq!(cpu.registers[0x1], 0x10 / 2);
        }

        //8xy7
        #[test]
        fn it_subtracts_vx_from_vy() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x10; //Vx
            cpu.registers[0x5] = 0x1C; //Vy
            cpu.run_opcode(0x8157);
            assert_eq!(cpu.registers[0x1], 0x1C - 0x10);
            assert_eq!(cpu.registers[0xF], 1);
        }

        //8xy7
        #[test]
        fn it_carries_subtracts_vx_from_vy() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0x1C; //Vx
            cpu.registers[0x5] = 0x10; //Vy
            cpu.run_opcode(0x8157);
            assert_eq!(cpu.registers[0x1], (0x110 - 0x1C) as u8);
            assert_eq!(cpu.registers[0xF], 0);
        }

        //8xyE
        #[test]
        fn it_sets_vf_if_msb_1() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0xFF; //Vx
            cpu.run_opcode(0x815E);
            assert_eq!(cpu.registers[0xF], 1);
            assert_eq!(cpu.registers[0x1], (0xFF * 2u16 & 0xFF) as u8);
        }

        //8xyE
        #[test]
        fn it_unsets_vf_if_msb_0() {
            let mut cpu = Cpu::new();
            cpu.registers[0x1] = 0b0100_0000; //Vx
            cpu.run_opcode(0x815E);
            assert_eq!(cpu.registers[0xF], 0);
            assert_eq!(cpu.registers[0x1], 0x40 * 2);
        }

        //9xy0
        #[test]
        fn it_skips_if_registers_not_equal() {
            let mut cpu = Cpu::new();
            cpu.registers[0xF] = 0xAB;
            cpu.registers[0xD] = 0xAA;
            cpu.pc = 10;
            cpu.run_opcode(0x9FD0);
            assert_eq!(12, cpu.pc);
        }

        //Annn
        #[test]
        fn it_sets_i_to_nnn() {
            let mut cpu = Cpu::new();
            cpu.run_opcode(0xA123);
            assert_eq!(cpu.i, 0x123);
        }

        //Bnnn
        #[test]
        fn it_jumps_to_nnn_plus_v0() {
            let mut cpu = Cpu::new();
            cpu.registers[0] = 0x12;
            cpu.run_opcode(0xB123);
            assert_eq!(cpu.pc, 0x123 + 0x12);
        }

        //Cxkk
        #[test]
        fn it_ands_00_with_rand_and_stores_0() {
            let mut cpu = Cpu::new();
            cpu.registers[0xC] = 0x12;
            cpu.run_opcode(0xCC00);
            assert_eq!(cpu.registers[0xC], 0);
        }
    }
}
