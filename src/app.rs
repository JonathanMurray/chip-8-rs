use crate::chip8::Chip8;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawParam, FilterMode, Font, Image, Text};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameError, GameResult};
use mint::Point2;

const COLOR_HIGHLIGHT: Color = Color::new(0.4, 1.0, 0.5, 1.0);
const COLOR_BG: Color = Color::new(0.2, 0.2, 0.3, 1.0);
const SCALING: f32 = 8.0;
const C8_WIDTH: u8 = 64;
const C8_HEIGHT: u8 = 32;
const DEBUG_Y_OFFSET: u32 = C8_HEIGHT as u32 * SCALING as u32;
const DEBUG_HEIGHT: u32 = 255;
const INSTRUCTION_LISTING_X_OFFSET: u32 = C8_WIDTH as u32 * SCALING as u32;
const INSTRUCTION_LISTING_WIDTH: u32 = 200;
const INSTRUCTION_LISTING_LENGTH: u32 = 32;

pub fn run(
    chip8: Chip8,
    disassembled_program: Vec<String>,
    window_title: String,
) -> Result<(), GameError> {
    let debug = true;
    let window_width;
    let window_height;
    if debug {
        window_width = C8_WIDTH as f32 * SCALING + INSTRUCTION_LISTING_WIDTH as f32;
        window_height = C8_HEIGHT as f32 * SCALING + DEBUG_HEIGHT as f32;
    } else {
        window_width = C8_WIDTH as f32 * SCALING;
        window_height = C8_HEIGHT as f32 * SCALING;
    }
    let (mut ctx, mut event_loop) = ContextBuilder::new("ggez_test", "jm")
        .window_setup(WindowSetup::default().title(&window_title))
        .window_mode(WindowMode::default().dimensions(window_width, window_height))
        .add_resource_path(".")
        .build()
        .expect("Creating ggez context");

    let mut app = App::new(&mut ctx, chip8, disassembled_program, debug, window_title)?;
    event::run(&mut ctx, &mut event_loop, &mut app)
}

struct App {
    font: Font,
    c8_screen_buffer: [u8; 4 * C8_WIDTH as usize * C8_HEIGHT as usize],
    chip8: Chip8,
    disassembled_program: Vec<String>,
    debug: bool,
    paused: bool,
    instruction_listing: Vec<(usize, String)>,
    cycles: u32,
    fast_forwarded_cycles: u32,
    window_title: String,
}

impl App {
    pub fn new(
        ctx: &mut Context,
        chip8: Chip8,
        disassembled_program: Vec<String>,
        debug: bool,
        window_title: String,
    ) -> GameResult<App> {
        let font = Font::new(ctx, "/Merchant Copy.ttf")?;
        let c8_screen_buffer = [255; 4 * C8_WIDTH as usize * C8_HEIGHT as usize];
        let app = App {
            font: font,
            c8_screen_buffer: c8_screen_buffer,
            chip8: chip8,
            disassembled_program: disassembled_program,
            debug: debug,
            paused: false,
            instruction_listing: vec![(0, String::new()); INSTRUCTION_LISTING_LENGTH as usize],
            cycles: 0,
            fast_forwarded_cycles: 0,
            window_title: window_title,
        };
        Ok(app)
    }

    fn draw_text(&self, ctx: &mut Context, s: &str, x: f32, y: f32) -> GameResult<()> {
        let text = Text::new((s, self.font, 25.0));
        graphics::draw(
            ctx,
            &text,
            DrawParam::default()
                .scale([0.5, 0.5])
                .dest(Point2 { x: x, y: y }),
        )
    }

    fn draw_text_with_color(
        &self,
        ctx: &mut Context,
        s: &str,
        x: f32,
        y: f32,
        color: Color,
    ) -> GameResult<()> {
        let text = Text::new((s, self.font, 25.0));
        graphics::draw(
            ctx,
            &text,
            DrawParam::default()
                .scale([0.5, 0.5])
                .dest(Point2 { x: x, y: y })
                .color(color),
        )
    }

    fn draw_debug_area(&mut self, ctx: &mut Context) -> GameResult<()> {
        let line_height = 15.0;
        let margin = 10.0;

        for (i, register_value) in self.chip8.registers.iter().enumerate() {
            self.draw_text(
                ctx,
                &format!("V{:X}: {:02X}", i, register_value),
                margin,
                DEBUG_Y_OFFSET as f32 + margin + i as f32 * line_height,
            )?;
        }

        let x = 80.0;
        let mut y = DEBUG_Y_OFFSET as f32 + margin;
        self.draw_text(
            ctx,
            &format!("I: {:04X}", self.chip8.address_register),
            x,
            y,
        )?;

        y += line_height;
        self.draw_text(
            ctx,
            &format!("PC: {:03X}", self.chip8.program_counter),
            x,
            y,
        )?;

        y += line_height;
        self.draw_text(
            ctx,
            &format!("Delay timer: {:02X}", self.chip8.delay_timer),
            x,
            y,
        )?;

        y += line_height;
        self.draw_text(
            ctx,
            &format!("Sound timer: {:02X}", self.chip8.sound_timer),
            x,
            y,
        )?;

        y += line_height * 2.0;
        self.draw_text(ctx, &"Stack:", x, y)?;
        for i in 0..self.chip8.stack_pointer {
            self.draw_text(
                ctx,
                &format!("{:03X}", self.chip8.stack[i as usize]),
                x + 50.0 + i as f32 * 28.0,
                y,
            )?;
        }

        y += line_height * 2.0;
        self.draw_text(ctx, "Next instruction:", x, y)?;
        let text = format!(
            "{}",
            match self
                .disassembled_program
                .get(self.chip8.program_counter as usize)
            {
                Some(s) => s,
                None => "?",
            }
        );
        self.draw_text_with_color(ctx, &text, x + 120.0, y, COLOR_HIGHLIGHT)?;

        y += line_height * 2.0;
        self.draw_text(
            ctx,
            &format!("Clock frequency: {}", self.chip8.clock_frequency()),
            x,
            y,
        )?;

        y += line_height * 2.0;
        self.draw_text(
            ctx,
            &format!("Status: {}", if self.paused { "PAUSED" } else { "RUNNING" }),
            x,
            y,
        )?;

        y += line_height * 2.0;
        self.draw_text(ctx, &format!("Cycles: {}", self.cycles), x, y)?;
        y += line_height;
        self.draw_text(
            ctx,
            &format!("Fast-forwarded cycles: {}", self.fast_forwarded_cycles),
            x,
            y,
        )?;

        Ok(())
    }

    fn draw_instruction_listing(&mut self, ctx: &mut Context) -> GameResult<()> {
        let line_height = 15.0;
        let margin = 15.0;
        let pc = self.chip8.program_counter as usize;
        if pc < self.instruction_listing[0].0
            || pc > self.instruction_listing[INSTRUCTION_LISTING_LENGTH as usize - 1].0
        {
            let mut address = (pc as f32 / INSTRUCTION_LISTING_LENGTH as f32) as usize
                * INSTRUCTION_LISTING_LENGTH as usize;
            let mut i = 0;
            while address < 0x1000 && i < INSTRUCTION_LISTING_LENGTH as usize {
                let text = self
                    .disassembled_program
                    .get(address)
                    .expect("Get disassembled");
                if !text.is_empty() {
                    self.instruction_listing[i] = (address, text.clone());
                    i += 1;
                }
                address += 1;
            }
            while i < INSTRUCTION_LISTING_LENGTH as usize {
                self.instruction_listing[i] = (usize::MAX, String::new());
                i += 1;
            }
        }

        let x = INSTRUCTION_LISTING_X_OFFSET as f32 + margin;
        for (i, (address, text)) in self.instruction_listing.iter().enumerate() {
            if address != &usize::MAX {
                let y = margin + i as f32 * line_height;
                let line = format!("{:03X}: {}", address, text);
                if &pc == address {
                    self.draw_text_with_color(ctx, &line, x, y, COLOR_HIGHLIGHT)?;
                } else {
                    self.draw_text(ctx, &line, x, y)?;
                }
            }
        }
        Ok(())
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.paused {
            let dt = timer::delta(ctx).as_secs_f64();
            let cycles = self.chip8.update(dt).expect("chip8 update");
            self.cycles += cycles;
            if cycles > 1 {
                self.fast_forwarded_cycles += cycles - 1;
            }
        }

        let fps = timer::fps(ctx) as u32;
        graphics::set_window_title(ctx, &format!("[{}]    (FPS: {})", self.window_title, fps));

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, COLOR_BG);

        for y in 0..C8_HEIGHT {
            for x in 0..C8_WIDTH {
                let offset = 4 * (y as usize * C8_WIDTH as usize + x as usize);
                if self.chip8.display_buffer.get_pixel(x, y) {
                    self.c8_screen_buffer[offset] = 255;
                    self.c8_screen_buffer[offset + 1] = 255;
                    self.c8_screen_buffer[offset + 2] = 255;
                } else {
                    self.c8_screen_buffer[offset] = 0;
                    self.c8_screen_buffer[offset + 1] = 0;
                    self.c8_screen_buffer[offset + 2] = 0;
                }
            }
        }

        let mut c8_screen_image = Image::from_rgba8(
            ctx,
            C8_WIDTH as u16,
            C8_HEIGHT as u16,
            &self.c8_screen_buffer,
        )?;
        c8_screen_image.set_filter(FilterMode::Nearest);
        graphics::draw(
            ctx,
            &c8_screen_image,
            DrawParam::default().scale([SCALING as f32, SCALING as f32]),
        )?;

        if self.debug {
            self.draw_debug_area(ctx)?;
            self.draw_instruction_listing(ctx)?;
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
            c8_handle_key(&mut self.chip8, keycode, true);

            match keycode {
                KeyCode::Escape => ggez::event::quit(ctx),
                KeyCode::P => self.chip8.multiply_clock_frequency(1.25),
                KeyCode::O => self.chip8.multiply_clock_frequency(0.8),
                KeyCode::Return => self.paused = !self.paused,
                KeyCode::L => self.debug = !self.debug,
                _ => {}
            }
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        c8_handle_key(&mut self.chip8, keycode, false);
    }
}

fn c8_handle_key(chip8: &mut Chip8, keycode: KeyCode, pressed: bool) {
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
