use chip_8_rs::chip8::{Chip8, FONT_SPRITES};
use chip_8_rs::{app, assembly};

use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let (filename, clock_frequency) = parse_args();

    let (mut chip8, disassembled_program) = setup_chip8(&filename);

    if let Some(freq) = clock_frequency {
        chip8.set_clock_frequency(freq);
        println!("Running {} at {} Hz", filename, freq);
    } else {
        println!("Running {}", filename);
    }

    app::run(chip8, disassembled_program, &filename).expect("Run app");
}

fn parse_args() -> (String, Option<u32>) {
    let mut args: Vec<String> = env::args().collect();
    let filename: String;
    let mut clock_frequency = None;
    match args.len() {
        1 => {
            filename = "Space Invaders [David Winter].ch8".to_string();
        }
        2 => {
            filename = args.remove(1);
        }
        3 => {
            filename = args.remove(1);
            let clock_frequency_arg = &args[1];
            match clock_frequency_arg.parse::<u32>() {
                Ok(freq) => {
                    clock_frequency = Some(freq);
                }
                Err(err) => {
                    println!(
                        "Invalid non-integer clock frequency: {} ({})",
                        clock_frequency_arg, err
                    );
                    println!("Usage: {} [ filename [clock_frequency] ]", args[0]);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            println!("Usage: {} [ filename [clock_frequency] ]", args[0]);
            std::process::exit(1);
        }
    }
    (filename, clock_frequency)
}

fn setup_chip8(filename: &str) -> (Chip8, Vec<String>) {
    let mut f = File::open(filename).expect("Opening ROM file");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading from ROM file");
    let mut memory = [0; 0x1000];
    for i in 0..buffer.len() {
        memory[0x200 + i] = buffer[i];
    }

    let disassembled_program = assembly::disassemble_rom(buffer);

    for i in 0..FONT_SPRITES.len() {
        memory[i] = FONT_SPRITES[i];
    }
    (Chip8::new(memory), disassembled_program)
}
