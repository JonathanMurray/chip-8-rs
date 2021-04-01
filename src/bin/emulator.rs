use chip_8_rs::chip8::{Chip8, FONT_SPRITES};
use chip_8_rs::{app, assembly};

use std::fs::File;
use std::io::Read;

use clap::{App, Arg};

fn main() {
    let (filename, clock_frequency, debug) = parse_args();

    let (mut chip8, disassembled_program) = setup_chip8(&filename);

    if let Some(freq) = clock_frequency {
        chip8.set_clock_frequency(freq);
        println!("Running {} at {} Hz", filename, freq);
    } else {
        println!("Running {}", filename);
    }

    app::run(chip8, disassembled_program, filename, debug).expect("Run app");
}

fn parse_args() -> (String, Option<u32>, bool) {
    let matches = App::new("Chip-8 emulator")
        .version("0.1.0")
        .about("An emulator/debugger of the virtual machine Chip-8, programmed in Rust.")
        .arg(
            Arg::with_name("ROM_FILE")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("A file containing the program that will be run"),
        )
        .arg(
            Arg::with_name("CLOCK_FREQUENCY")
                .short("c")
                .long("clock")
                .takes_value(true)
                .help("The number of instructions to be executed by Chip-8 per second"),
        )
        .arg(
            Arg::with_name("DEBUG")
                .short("d")
                .long("debug")
                .help("Show debug information (like register contents and disassembled instructions) while running"),
        )
        .get_matches();

    let filename = matches
        .value_of("ROM_FILE")
        .unwrap_or("programs/Space Invaders [David Winter].ch8")
        .to_owned();

    let clock_frequency = match matches.value_of("CLOCK_FREQUENCY") {
        Some(freq) => match freq.parse::<u32>() {
            Ok(freq) => Some(freq),
            Err(err) => {
                panic!("Invalid non-integer clock frequency: {} ({})", freq, err);
            }
        },
        None => None,
    };

    let debug = matches.occurrences_of("DEBUG") > 0;

    (filename, clock_frequency, debug)
}

fn setup_chip8(filename: &str) -> (Chip8, Vec<String>) {
    let mut f = File::open(filename).expect(&format!("Couldn't open ROM file: {}", filename));
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .expect(&format!("Couldn't read from ROM file: {}", filename));
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
