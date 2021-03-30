use crate::chip8::Chip8;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, DrawParam, FilterMode, Font, Image, Text};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameError, GameResult};
use mint::Point2;

const SCALING: f32 = 8.0;
const C8_WIDTH: u8 = 64;
const C8_HEIGHT: u8 = 32;
const DEBUG_MARGIN: u32 = 10;
const DEBUG_Y_OFFSET: u32 = C8_HEIGHT as u32 * SCALING as u32 + DEBUG_MARGIN;
const DEBUG_HEIGHT: u32 = 240;

pub fn run(chip8: Chip8, window_title: &str) -> Result<(), GameError> {
    let debug = true;
    let window_width = C8_WIDTH as f32 * SCALING;
    let window_height;
    if debug {
        window_height = C8_HEIGHT as f32 * SCALING + DEBUG_MARGIN as f32 + DEBUG_HEIGHT as f32;
    } else {
        window_height = C8_HEIGHT as f32 * SCALING;
    }
    let (mut ctx, mut event_loop) = ContextBuilder::new("ggez_test", "jm")
        .window_setup(WindowSetup::default().title(window_title))
        .window_mode(WindowMode::default().dimensions(window_width, window_height))
        .add_resource_path(".")
        .build()
        .expect("Creating ggez context");

    let mut app = App::new(&mut ctx, chip8, debug)?;
    event::run(&mut ctx, &mut event_loop, &mut app)
}

struct App {
    font: Font,
    image_buffer: [u8; 4 * C8_WIDTH as usize * C8_HEIGHT as usize],
    chip8: Chip8,
    debug: bool,
}

impl App {
    pub fn new(ctx: &mut Context, chip8: Chip8, debug: bool) -> GameResult<App> {
        let font = Font::new(ctx, "/Merchant Copy.ttf")?;
        let image_buffer = [255; 4 * C8_WIDTH as usize * C8_HEIGHT as usize];
        let app = App {
            font: font,
            image_buffer: image_buffer,
            chip8: chip8,
            debug: debug,
        };
        Ok(app)
    }

    fn draw_debug_area(&mut self, ctx: &mut Context) -> GameResult<()> {
        let font_size = 12.5;
        let line_height = 14.5;

        for (i, register_value) in self.chip8.registers.iter().enumerate() {
            let text = Text::new((
                format!("V{:X}: {:02X}", i, register_value),
                self.font,
                font_size,
            ));
            let text_pos = Point2 {
                x: 10.0,
                y: DEBUG_Y_OFFSET as f32 + i as f32 * line_height,
            };
            graphics::draw(ctx, &text, DrawParam::default().dest(text_pos))?;
        }

        let text = Text::new((
            format!("I: {:04X}", self.chip8.address_register),
            self.font,
            font_size,
        ));
        let text_pos = Point2 {
            x: 80.0,
            y: DEBUG_Y_OFFSET as f32,
        };
        graphics::draw(ctx, &text, DrawParam::default().dest(text_pos))?;

        let text = Text::new((
            format!("PC: {:04X}", self.chip8.program_counter),
            self.font,
            font_size,
        ));
        let text_pos = Point2 {
            x: 80.0,
            y: DEBUG_Y_OFFSET as f32 + line_height,
        };
        graphics::draw(ctx, &text, DrawParam::default().dest(text_pos))?;

        let text = Text::new((
            format!("Delay timer: {:02X}", self.chip8.delay_timer),
            self.font,
            font_size,
        ));
        let text_pos = Point2 {
            x: 80.0,
            y: DEBUG_Y_OFFSET as f32 + line_height * 2.0,
        };
        graphics::draw(ctx, &text, DrawParam::default().dest(text_pos))?;

        let text = Text::new((
            format!("Sound timer: {:02X}", self.chip8.sound_timer),
            self.font,
            font_size,
        ));
        let text_pos = Point2 {
            x: 80.0,
            y: DEBUG_Y_OFFSET as f32 + line_height * 3.0,
        };
        graphics::draw(ctx, &text, DrawParam::default().dest(text_pos))?;

        let text = Text::new(("Stack:", self.font, font_size));
        let text_pos = Point2 {
            x: 80.0,
            y: DEBUG_Y_OFFSET as f32 + line_height * 4.0,
        };
        graphics::draw(ctx, &text, DrawParam::default().dest(text_pos))?;
        for i in 0..self.chip8.stack_pointer + 1 {
            let text = Text::new((
                format!("{:04X}", self.chip8.stack[i as usize]),
                self.font,
                font_size,
            ));
            let text_pos = Point2 {
                x: 130.0 + i as f32 * 40.0,
                y: DEBUG_Y_OFFSET as f32 + line_height * 4.0,
            };
            graphics::draw(ctx, &text, DrawParam::default().dest(text_pos))?;
        }

        Ok(())
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::delta(ctx).as_secs_f64();

        self.chip8.update(dt).expect("chip8 update");

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb(120, 120, 120));

        for y in 0..C8_HEIGHT {
            for x in 0..C8_WIDTH {
                let offset = 4 * (y as usize * C8_WIDTH as usize + x as usize);
                if self.chip8.display_buffer.get_pixel(x, y) {
                    self.image_buffer[offset] = 255;
                    self.image_buffer[offset + 1] = 255;
                    self.image_buffer[offset + 2] = 255;
                } else {
                    self.image_buffer[offset] = 0;
                    self.image_buffer[offset + 1] = 0;
                    self.image_buffer[offset + 2] = 0;
                }
            }
        }

        let mut image =
            Image::from_rgba8(ctx, C8_WIDTH as u16, C8_HEIGHT as u16, &self.image_buffer)?;
        image.set_filter(FilterMode::Nearest);

        graphics::draw(
            ctx,
            &image,
            DrawParam::default().scale([SCALING as f32, SCALING as f32]),
        )?;

        if self.debug {
            self.draw_debug_area(ctx)?;
        }

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        repeat: bool,
    ) {
        if !repeat {
            handle_key(&mut self.chip8, keycode, true);

            if keycode == KeyCode::Escape {
                ggez::event::quit(ctx);
            }
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        handle_key(&mut self.chip8, keycode, false);
    }
}

fn handle_key(chip8: &mut Chip8, keycode: KeyCode, pressed: bool) {
    match keycode {
        KeyCode::Key0 => chip8.handle_key_event(0x0, pressed),
        KeyCode::Key1 => chip8.handle_key_event(0x1, pressed),
        KeyCode::Key2 => chip8.handle_key_event(0x2, pressed),
        KeyCode::Key3 => chip8.handle_key_event(0x3, pressed),
        KeyCode::Key4 => chip8.handle_key_event(0x4, pressed),
        KeyCode::Key5 => chip8.handle_key_event(0x5, pressed),
        KeyCode::Key6 => chip8.handle_key_event(0x6, pressed),
        KeyCode::Key7 => chip8.handle_key_event(0x7, pressed),
        KeyCode::Key8 => chip8.handle_key_event(0x8, pressed),
        KeyCode::Key9 => chip8.handle_key_event(0x9, pressed),
        KeyCode::A => chip8.handle_key_event(0xA, pressed),
        KeyCode::B => chip8.handle_key_event(0xB, pressed),
        KeyCode::C => chip8.handle_key_event(0xC, pressed),
        KeyCode::D => chip8.handle_key_event(0xD, pressed),
        KeyCode::E => chip8.handle_key_event(0xE, pressed),
        KeyCode::F => chip8.handle_key_event(0xF, pressed),
        _ => {}
    }
}
