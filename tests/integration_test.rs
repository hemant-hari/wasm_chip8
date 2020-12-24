use std::fs::File;
use std::io;
use std::io::prelude::*;
use wasm_chip8::chip8::Cpu;

#[test]
fn load() -> io::Result<()> {
    let mut cpu = Cpu::new();
    let mut f = File::open("C:\\Users\\Hemant Hari\\Downloads\\c8games\\HIDDEN")?;
    //let mut f = File::open("C:\\Users\\Hemant Hari\\Downloads\\BC_test.ch8")?;
    let mut buffer = [0; 4096];

    f.read(&mut buffer[0x200..])?;

    cpu.load_memory(buffer);

    for _ in 0..3000 {
        cpu.execute_cycle();
    }

    cpu.print_display();

    assert!(true);

    Ok(())
}
