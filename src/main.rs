use rust_chip_8::{Machine, FONT_SPRITES};
use std::env;
use std::fs::File;
use std::io::Read;

extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use image::{ImageBuffer, Rgba};
use opengl_graphics::OpenGL;
use piston::event_loop::{EventSettings, Events};
use piston::input::{PressEvent, ReleaseEvent, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use piston_window::{
    Button, Filter, G2dTexture, Key, PistonWindow, Texture, TextureContext, TextureSettings,
};

const SCALING: u32 = 8;

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

    let mut window: PistonWindow = WindowSettings::new(window_title, [64 * SCALING, 32 * SCALING])
        .graphics_api(OpenGL::V3_2)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let raw_image_buf = vec![0; 4 * 64 as usize * 32 as usize];
    let mut image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(64, 32, raw_image_buf).unwrap();
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let mut texture: G2dTexture = Texture::from_image(
        &mut texture_context,
        &image_buffer,
        &TextureSettings::new().filter(Filter::Nearest),
    )
    .unwrap();

    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(_render_args) = e.render_args() {
            use crate::graphics::Transformed;

            for y in 0..32 {
                for x in 0..64 {
                    if machine.display_buffer.get_pixel(x, y) {
                        image_buffer.put_pixel(x as u32, y as u32, Rgba([255, 255, 255, 255]));
                    } else {
                        image_buffer.put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 255]));
                    }
                }
            }

            texture.update(&mut texture_context, &image_buffer).unwrap();
            window.draw_2d(&e, |c, g, device| {
                texture_context.encoder.flush(device);
                graphics::image(
                    &texture,
                    c.transform.scale(SCALING as f64, SCALING as f64),
                    g,
                );
            });
        }

        if let Some(press_args) = e.press_args() {
            if let Button::Keyboard(key) = press_args {
                handle_key(&mut machine, key, true);
            }
        }

        if let Some(release_args) = e.release_args() {
            if let Button::Keyboard(key) = release_args {
                handle_key(&mut machine, key, false);
            }
        }

        if let Some(update_args) = e.update_args() {
            machine.update(update_args.dt).expect("Machine update");
        }
    }
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

fn handle_key(machine: &mut Machine, key: Key, pressed: bool) {
    match key {
        Key::D0 => machine.handle_key_event(0x0, pressed),
        Key::D1 => machine.handle_key_event(0x1, pressed),
        Key::D2 => machine.handle_key_event(0x2, pressed),
        Key::D3 => machine.handle_key_event(0x3, pressed),
        Key::D4 => machine.handle_key_event(0x4, pressed),
        Key::D5 => machine.handle_key_event(0x5, pressed),
        Key::D6 => machine.handle_key_event(0x6, pressed),
        Key::D7 => machine.handle_key_event(0x7, pressed),
        Key::D8 => machine.handle_key_event(0x8, pressed),
        Key::D9 => machine.handle_key_event(0x9, pressed),
        Key::A => machine.handle_key_event(0xA, pressed),
        Key::B => machine.handle_key_event(0xB, pressed),
        Key::C => machine.handle_key_event(0xC, pressed),
        Key::D => machine.handle_key_event(0xD, pressed),
        Key::E => machine.handle_key_event(0xE, pressed),
        Key::F => machine.handle_key_event(0xF, pressed),
        _ => {}
    }
}
