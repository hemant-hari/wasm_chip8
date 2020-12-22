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
    use rand::Rng;
    use wasm_bindgen::prelude::*;

    use crate::display::Display;
    use crate::display::Pixel;
    use crate::keyboard::Keyboard;

    #[wasm_bindgen]
    pub struct Cpu {
        i: u16,
        pc: u16,
        s_ptr: u8,
        stack: [u16; 16],
        delay_timer: u8,
        sound_timer: u8,
        registers: [u8; 16],
        memory: [u8; 4096],
        display: Display,
        keyboard: Keyboard,
    }

    #[wasm_bindgen]
    impl Cpu {
        pub fn new() -> Cpu {
            Cpu {
                i: 0,
                pc: 0,
                s_ptr: 0,
                stack: [0; 16],
                delay_timer: 0,
                sound_timer: 0,
                registers: [0; 16],
                memory: [0; 4096],
                display: Display::new_empty(),
                keyboard: Keyboard::new(),
            }
        }

        pub fn get_memory(&self) -> *const u8 {
            self.memory.as_ptr()
        }

        pub fn get_display(&self) -> *const Pixel {
            self.display.pixels()
        }

        pub fn set_keys(&mut self, keys_binary: u16) {
            self.keyboard.set_keys(keys_binary);
        }

        pub fn execute_cycle(&mut self) {
            let opcode = (self.memory[self.pc as usize] as u16) << 8
                | (self.memory[(self.pc + 1) as usize] as u16);
            self.run_opcode(opcode);
        }

        fn run_opcode(&mut self, opcode: u16) {
            let nibbles = Cpu::get_nibbles(opcode);
            let nnn = || opcode & 0x0FFF;
            let kk = || (opcode & 0x00FF) as u8;

            match nibbles {
                (0, 0, 0xE, 0) => self.display.cls(),
                (0, 0, 0xE, 0xE) => self.ret_subroutine(),
                (1, _, _, _) => self.pc = nnn(),
                (2, _, _, _) => self.call_subroutine(nnn()),
                (3, x, _, _) if self.registers[x] == kk() => self.pc += 2,
                (4, x, _, _) if self.registers[x] != kk() => self.pc += 2,
                (5, x, y, 0) if self.registers[x] == self.registers[y] => self.pc += 2,
                (6, x, _, _) => self.registers[x] = kk(),
                (7, x, _, _) => self.registers[x] += kk(),
                (8, x, y, 0) => self.registers[x] = self.registers[y],
                (8, x, y, 1) => self.registers[x] |= self.registers[y],
                (8, x, y, 2) => self.registers[x] &= self.registers[y],
                (8, x, y, 3) => self.registers[x] ^= self.registers[y],
                (8, x, y, 4) => self.registers[x] = self.safe_add_registers(x, y),
                (8, x, y, 5) => self.registers[x] = self.safe_sub_registers(x, y),
                (8, x, _, 6) => self.registers[x] = self.halve(x),
                (8, x, y, 7) => self.registers[x] = self.safe_sub_registers(y, x),
                (8, x, _, 0xE) => self.registers[x] = self.double(x),
                (9, x, y, 0) if self.registers[x as usize] != self.registers[y] => self.pc += 2,
                (0xA, _, _, _) => self.i = nnn(),
                (0xB, _, _, _) => self.pc = nnn() + self.registers[0] as u16,
                (0xC, x, _, _) => self.registers[x] = kk() & rand::thread_rng().gen_range(0, 0xFF),
                (0xD, x, y, n) => self.display_sprite(self.registers[x], self.registers[y], n), //TODO
                (0xE, x, 9, 0xE) if self.keyboard.key_is_pressed(x as u8) => self.pc += 2,
                (0xE, x, 0xA, 1) if !self.keyboard.key_is_pressed(x as u8) => self.pc += 2,
                (0xF, x, 0, 7) => self.registers[x] = self.delay_timer,
                (0xF, x, 0, 0xA) if !self.keyboard.key_is_pressed(x as u8) => self.pc -= 2,
                (0xF, x, 1, 5) => self.delay_timer = self.registers[x],
                (0xF, x, 1, 8) => self.sound_timer = self.registers[x],
                (0xF, x, 1, 0xE) => self.i += self.registers[x] as u16,
                (0xF, x, 2, 9) => self.i = self.registers[x] as u16 * 5,
                (0xF, x, 3, 3) => self.write_bcd_to_memory(self.registers[x], self.i as usize),
                (0xF, x, 5, 5) => self.store_registers(x as usize, self.i as usize),
                (0xF, x, 6, 5) => self.load_registers(x as usize, self.i as usize),
                _ => return,
            }
        }

        fn get_nibbles(opcode: u16) -> (u8, usize, usize, u8) {
            (
                ((opcode & 0xF000) >> 12) as u8,
                ((opcode & 0x0F00) >> 8) as usize,
                ((opcode & 0x00F0) >> 4) as usize,
                (opcode & 0x000F) as u8,
            )
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
            let (sum, overflow) = self.registers[x].overflowing_add(self.registers[y]);
            self.registers[0xF] = overflow as u8;
            sum
        }

        fn safe_sub_registers(&mut self, first: usize, second: usize) -> u8 {
            let (diff, overflow) = self.registers[first].overflowing_sub(self.registers[second]);
            self.registers[0xF] = !overflow as u8;
            diff
        }

        fn halve(&mut self, x: usize) -> u8 {
            self.registers[0xF] = self.registers[x] & 1;
            self.registers[x] >> 1
        }

        fn double(&mut self, x: usize) -> u8 {
            self.registers[0xF] = self.registers[x] & 0b1000_0000 >> 7;
            self.registers[x] << 1
        }

        fn display_sprite(&mut self, x: u8, y: u8, bytes: u8) {
            let i = self.i as usize;
            self.display
                .draw_bytes(x, y, &self.memory[i..i + bytes as usize])
        }

        fn write_bcd_to_memory(&mut self, value: u8, address: usize){
            self.memory[address] = (value/100)%10;
            self.memory[address+1] = (value/10)%10;
            self.memory[address+2] = value%10;
        }

        fn store_registers(&mut self, upto: usize, address: usize){
            for i in 0..upto+1 {
                self.memory[address + i] = self.registers[i];
            }
        }

        fn load_registers(&mut self, upto: usize, address: usize){
            for i in 0..upto+1 {
                self.registers[i] = self.memory[address + i];
            }
        }
    }

    #[cfg(test)]
    mod cpu_tests {
        use super::*;
        use crate::display::Pixel;

        //00E0
        #[test]
        fn it_clears_screen() {
            let mut cpu = Cpu::new();
            cpu.display.toggle_pixel(1);
            cpu.run_opcode(0x00E0);
            assert!(cpu.display.get_pixel(1) == Pixel::Off);
        }

        #[test]
        fn it_splits_nibbles() {
            let nibbles = Cpu::get_nibbles(0x1234);
            let expected_nibbles: (u8, usize, usize, u8) = (1, 2, 3, 4);
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

        //Dxyn
        #[test]
        fn it_displays_sprite() {
            let mut cpu = Cpu::new();
            cpu.registers[0] = 0;
            cpu.registers[1] = 0;
            cpu.memory[0] = 0b1000_0001;
            cpu.memory[1] = 0b1000_0001;
            cpu.memory[2] = 0b1111_1111;
            cpu.memory[3] = 0b1000_0001;
            cpu.memory[4] = 0b1000_0001;
            cpu.run_opcode(0xD015);
            cpu.registers[0] = 9;
            cpu.run_opcode(0xD015);
            println!("{}", cpu.display.to_string());
        }

        //Ex9E
        #[test]
        fn it_skips_instruction_if_key_pressed() {
            let mut cpu = Cpu::new();
            cpu.keyboard.set_key(0xC);
            cpu.pc = 10;
            cpu.run_opcode(0xEC9E);
            assert_eq!(cpu.pc, 12);
        }

        //ExA1
        #[test]
        fn it_skips_instruction_if_key_not_pressed() {
            let mut cpu = Cpu::new();
            cpu.keyboard.set_key(0xB);
            cpu.pc = 10;
            cpu.run_opcode(0xECA1);
            assert_eq!(cpu.pc, 12);
        }

        //Fx07
        #[test]
        fn it_load_delay_timer_value() {
            let mut cpu = Cpu::new();
            cpu.delay_timer = 0x12;
            cpu.run_opcode(0xFF07);
            assert_eq!(cpu.registers[0xF], 0x12);
        }

        //Fx0A
        #[test]
        fn it_waits_for_key_press() {
            let mut cpu = Cpu::new();
            cpu.pc = 2;
            cpu.run_opcode(0xF10A);
            assert_eq!(cpu.pc, 0);
        }

        //Fx15
        #[test]
        fn it_sets_delay_timer() {
            let mut cpu = Cpu::new();
            cpu.registers[0x5] = 0x12;
            cpu.run_opcode(0xF515);
            assert_eq!(cpu.delay_timer, 0x12);
        }

        //Fx18
        #[test]
        fn it_sets_sound_timer() {
            let mut cpu = Cpu::new();
            cpu.registers[0x5] = 0x12;
            cpu.run_opcode(0xF518);
            assert_eq!(cpu.sound_timer, 0x12);
        }

        //Fx1E
        #[test]
        fn it_adds_vx_to_i() {
            let mut cpu = Cpu::new();
            cpu.i = 10;
            cpu.registers[0x1] = 10;
            cpu.run_opcode(0xF11E);
            assert_eq!(cpu.i, 20);
        }

        //Fx29
        #[test]
        fn it_loads_i_from_vx() {
            let mut cpu = Cpu::new();
            cpu.i = 10;
            cpu.registers[0x1] = 0x9;
            cpu.run_opcode(0xF129);
            assert_eq!(cpu.i, 0x9 * 5);
        }

        //Fx33
        #[test]
        fn it_stores_bcd_representation() {
            let mut cpu = Cpu::new();
            cpu.i = 0x400;
            cpu.registers[0x1] = 223;
            cpu.run_opcode(0xF133);
            assert_eq!(cpu.memory[0x400], 2);
            assert_eq!(cpu.memory[0x401], 2);
            assert_eq!(cpu.memory[0x402], 3);
        }

        //Fx55
        #[test]
        fn it_stores_all_registers() {
            let mut cpu = Cpu::new();
            cpu.i = 0x400;
            cpu.registers[0x0] = 0x11;
            cpu.registers[0x1] = 0x22;
            cpu.registers[0x2] = 0x33;
            cpu.registers[0x3] = 0x44;
            cpu.run_opcode(0xF355);
            assert_eq!(cpu.memory[0x400], 0x11u8);
            assert_eq!(cpu.memory[0x401], 0x22u8);
            assert_eq!(cpu.memory[0x402], 0x33u8);
            assert_eq!(cpu.memory[0x403], 0x44u8);
        }

        //Fx65
        #[test]
        fn it_loads_all_registers() {
            let mut cpu = Cpu::new();
            cpu.i = 0x400;
            cpu.memory[0x400] = 0x11;
            cpu.memory[0x401] = 0x22;
            cpu.memory[0x402] = 0x33;
            cpu.memory[0x403] = 0x44;
            cpu.run_opcode(0xF365);
            assert_eq!(cpu.registers[0x0], 0x11u8);
            assert_eq!(cpu.registers[0x1], 0x22u8);
            assert_eq!(cpu.registers[0x2], 0x33u8);
            assert_eq!(cpu.registers[0x3], 0x44u8);
        }
    }
}
