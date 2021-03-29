use crate::machine::Machine;

use graphics::text::Text;
use image::{ImageBuffer, Rgba};
use piston::event_loop::{EventSettings, Events};
use piston::input::{PressEvent, ReleaseEvent, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use piston_window::{
    Button, Filter, G2dTexture, Key, PistonWindow, Texture, TextureContext, TextureSettings,
};

const SCALING: u32 = 8;
const C8_WIDTH: u32 = 64;
const C8_HEIGHT: u32 = 32;
const DEBUG_MARGIN: u32 = 10;
const DEBUG_Y_OFFSET: u32 = C8_HEIGHT * SCALING + DEBUG_MARGIN;
const DEBUG_HEIGHT: u32 = 240;

pub fn run(mut machine: Machine, window_title: &str) {
    let mut window: PistonWindow = WindowSettings::new(
        window_title,
        [
            C8_WIDTH * SCALING,
            C8_HEIGHT * SCALING + DEBUG_MARGIN + DEBUG_HEIGHT,
        ],
    )
    .exit_on_esc(true)
    .samples(0)
    .build()
    .unwrap();

    let raw_image_buf = vec![0; 4 * C8_WIDTH as usize * C8_HEIGHT as usize];
    let mut image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(C8_WIDTH, C8_HEIGHT, raw_image_buf).unwrap();
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

    let font = "Merchant Copy.ttf";
    let mut glyphs = window.load_font(font).unwrap();

    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(_render_args) = e.render_args() {
            use graphics::Transformed;

            for y in 0..C8_HEIGHT as u8 {
                for x in 0..C8_WIDTH as u8 {
                    if machine.display_buffer.get_pixel(x, y) {
                        image_buffer.put_pixel(x as u32, y as u32, Rgba([255, 255, 255, 255]));
                    } else {
                        image_buffer.put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 255]));
                    }
                }
            }

            texture.update(&mut texture_context, &image_buffer).unwrap();
            window.draw_2d(&e, |c, g, device| {
                graphics::clear([0.3, 0.3, 0.3, 1.0], g);

                texture_context.encoder.flush(device);
                graphics::image(
                    &texture,
                    c.transform.scale(SCALING as f64, SCALING as f64),
                    g,
                );

                
                // We use a scaling-hack to get sharp text, as suggested here
                // https://github.com/PistonDevelopers/piston/issues/1240#issuecomment-569318143
                let mut text_transform = c.transform;
                text_transform = text_transform
                    .trans(20.0, (DEBUG_Y_OFFSET + 10) as f64)
                    .scale(0.5, 0.5);
                for (i, register_value) in machine.registers.iter().enumerate() {
                    Text::new_color([1.0, 1.0, 1.0, 1.0], 20)
                        .draw(
                            &format!("V{:X}: {:02X}", i, register_value),
                            &mut glyphs,
                            &c.draw_state,
                            text_transform.trans(0.0, (i * 27) as f64),
                            g,
                        )
                        .unwrap();
                }
                text_transform = text_transform.trans(160.0, 0.0);
                Text::new_color([1.0, 1.0, 1.0, 1.0], 20)
                    .draw(
                        &format!("I: {:04X}", machine.address_register),
                        &mut glyphs,
                        &c.draw_state,
                        text_transform,
                        g,
                    )
                    .unwrap();
                text_transform = text_transform.trans(0.0, 27.0);
                Text::new_color([1.0, 1.0, 1.0, 1.0], 20)
                    .draw(
                        &format!("PC: {:04X}", machine.program_counter),
                        &mut glyphs,
                        &c.draw_state,
                        text_transform,
                        g,
                    )
                    .unwrap();
                text_transform = text_transform.trans(0.0, 27.0);
                Text::new_color([1.0, 1.0, 1.0, 1.0], 20)
                    .draw(
                        &format!("Delay timer: {:02X}", machine.delay_timer),
                        &mut glyphs,
                        &c.draw_state,
                        text_transform,
                        g,
                    )
                    .unwrap();
                text_transform = text_transform.trans(0.0, 27.0);
                Text::new_color([1.0, 1.0, 1.0, 1.0], 20)
                    .draw(
                        &format!("Sound timer: {:02X}", machine.sound_timer),
                        &mut glyphs,
                        &c.draw_state,
                        text_transform,
                        g,
                    )
                    .unwrap();
                text_transform = text_transform.trans(0.0, 27.0);
                Text::new_color([1.0, 1.0, 1.0, 1.0], 20)
                    .draw("Stack:", &mut glyphs, &c.draw_state, text_transform, g)
                    .unwrap();
                for i in 0..machine.stack_pointer + 1 {
                    Text::new_color([1.0, 1.0, 1.0, 1.0], 20)
                        .draw(
                            &format!("{:04X}", machine.stack[i as usize]),
                            &mut glyphs,
                            &c.draw_state,
                            text_transform.trans(100.0 + i as f64 * 80.0, 0.0),
                            g,
                        )
                        .unwrap();
                }

                // Apparently we need to flush glyphs before rendering
                glyphs.factory.encoder.flush(device);
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
