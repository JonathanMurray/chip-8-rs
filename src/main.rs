use rust_chip_8::{Machine, FONT_SPRITES};
use std::fs::File;
use std::io::Read;

extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use image::{ImageBuffer, Rgba};
use opengl_graphics::OpenGL;
use piston::event_loop::{EventSettings, Events};
use piston::input::{PressEvent, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use piston_window::{
    Button, Filter, G2dTexture, Key, PistonWindow, Texture, TextureContext, TextureSettings,
};

const SCALING: u32 = 8;

fn setup_machine() -> Machine {
    //let mut f = File::open("test_opcode.ch8").expect("Open test file");
    let mut f = File::open("Pong (1 player).ch8").expect("Open test file");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Read from test file");
    let mut memory = [0; 0x1000];
    for (i, b) in buffer.into_iter().enumerate() {
        memory[0x200 + i] = b;
    }
    for i in 0..FONT_SPRITES.len() {
        memory[i] = FONT_SPRITES[i];
    }
    Machine::new(memory)
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("CHIP-8", [64 * SCALING, 32 * SCALING])
        .graphics_api(OpenGL::V3_2)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut machine = setup_machine();

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

    let mut cooldown = 0.0;

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
            match press_args {
                Button::Keyboard(Key::Space) => {
                    machine.step().expect("Machine step");
                }
                _ => {}
            }
        }

        if let Some(update_args) = e.update_args() {
            cooldown -= update_args.dt;
            if cooldown <= 0.0 {
                cooldown += 0.01;
                machine.step().expect("Machine step");
            }
        }
    }
}
