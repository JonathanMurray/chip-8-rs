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
    let mut f = File::open(filename).expect(&format!("Couldn't open ROM file: {}", filename));
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .expect(&format!("Couldn't read from ROM file: {}", filename));
    let mut memory = [0; 0x1000];
    for i in 0..buffer.len() {
        memory[0x200 + i] = buffer[i];
    }
    let disassembled_program = assembly::disassemble_rom(buffer);

    let mut output_file = File::create(&result_filename)
        .expect(&format!("Couldn't create output file: {}", result_filename));
    let mut num_instructions = 0;
    for (i, line) in disassembled_program.iter().enumerate() {
        if !line.is_empty() {
            writeln!(output_file, "{:03X}: {}", i, line).expect(&format!(
                "Couldn't write disassembled program to file: {}",
                result_filename
            ));
            num_instructions += 1;
        }
    }
    println!("Wrote {} instructions to {}", num_instructions, result_filename);
}
