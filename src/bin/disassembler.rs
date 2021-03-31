use chip_8_rs::assembly;

use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let (rom_file, result_file) = match args.len() {
        3 => (args.remove(1), args.remove(1)),
        _ => {
            println!("Usage: {} rom_filename result_file", args[0]);
            std::process::exit(1);
        }
    };

    disassemble(&rom_file, &result_file);
}

fn disassemble(filename: &str, result_filename: &str) {
    let mut f = File::open(filename).expect("Opening ROM file");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading from ROM file");
    let mut memory = [0; 0x1000];
    for i in 0..buffer.len() {
        memory[0x200 + i] = buffer[i];
    }
    let disassembled_program = assembly::disassemble_rom(buffer);

    let mut output_file = File::create(&result_filename).expect("Opening output file");
    for (i, line) in disassembled_program.iter().enumerate() {
        if !line.is_empty() {
            writeln!(output_file, "{:03X}: {}", i, line)
                .expect("Writing disassembled program to file");
        }
    }
}
