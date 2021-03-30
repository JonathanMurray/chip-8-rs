mod app;
mod machine;

use machine::{Machine, FONT_SPRITES};
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let (filename, clock_frequency) = parse_args();

    let mut machine = setup_machine(&filename);

    let window_title;
    if let Some(freq) = clock_frequency {
        machine.set_clock_frequency(freq);
        println!("Running {} at {} Hz", filename, freq);
        window_title = format!("{} ({}Hz)", filename, freq);
    } else {
        println!("Running {}", filename);
        window_title = filename;
    }

    app::run(machine, &window_title).expect("Run app");
}

fn parse_args() -> (String, Option<i32>) {
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
            match clock_frequency_arg.parse::<i32>() {
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

fn setup_machine(filename: &str) -> Machine {
    let mut f = File::open(filename).expect("Opening ROM file");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading from ROM file");
    let mut memory = [0; 0x1000];
    for (i, b) in buffer.into_iter().enumerate() {
        memory[0x200 + i] = b;
    }
    for i in 0..FONT_SPRITES.len() {
        memory[i] = FONT_SPRITES[i];
    }
    Machine::new(memory)
}
