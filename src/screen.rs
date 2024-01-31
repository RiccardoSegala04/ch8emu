use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use log::info;

const PIXEL_SHUTDOWN_FACTOR: u8 = 80;
const SCREEN_WIDTH: u32 = 64;
const SCREEN_HEIGHT: u32 = 32;
const BLOCK_SIZE: u32 = 12;
const WINDOW_WIDTH: u32 = SCREEN_WIDTH * BLOCK_SIZE + BLOCK_SIZE * 2;
const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT * BLOCK_SIZE + BLOCK_SIZE * 2;

// Represents the CHIP-8 screen
pub struct Screen {
    pixels: Vec<u8>,
    shutdown_pixels: Vec<u8>,
    keypad: Vec<bool>,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
}

impl Screen  {
    // Creates a new CHIP-8 screen with default values
    pub fn new() -> Screen {

        // Initialize SDL2
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        // Create a window
        let window = video_subsystem.window("CHIP-8 EMU", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        // Create a canvas from the window
        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        Screen {
            pixels: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
            shutdown_pixels: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
            keypad: vec![false; 16],
            canvas,
            event_pump: sdl_context.event_pump().unwrap(),
        }
    }

    // Clears the screen
    pub fn clear(&mut self) {
        self.pixels.iter_mut().for_each(|x| *x=0);
    }

    // Draws pixel buffer to the screen
    pub fn update(&mut self, draw: bool) {

        if draw || self.shutdown_pixels.iter().any(|x| *x > 0) {
            // Decrease the shutdown pixels
            self.shutdown_pixels.iter_mut().for_each(|x| *x = 
                x.saturating_sub(PIXEL_SHUTDOWN_FACTOR));
            // Draw the pixels
            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();
            for y in 0..32 {
                for x in 0..64 {
                    let i = y * 64 + x;
                    let pixel_rect = sdl2::rect::Rect::new(
                        (x as i32) * BLOCK_SIZE as i32 + BLOCK_SIZE as i32, 
                        (y as i32) * BLOCK_SIZE as i32 + BLOCK_SIZE as i32, 
                        BLOCK_SIZE, BLOCK_SIZE
                    );
                    if self.pixels[i] == 1 {
                        // Draw the pixel
                        self.canvas.set_draw_color(Color::WHITE);
                        self.canvas.fill_rect(pixel_rect).unwrap();
                    } else {
                        // Draw the shutdown pixel
                        let bright = self.shutdown_pixels[i];
                        self.canvas.set_draw_color(Color::RGB(bright, bright, bright));
                        self.canvas.fill_rect(pixel_rect).unwrap();
                    }
                }
            }
            // Present the canvas
            self.canvas.present();
        }
        
        // Handle events
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} => std::process::exit(0),
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    info!("Key pressed: {:?}", keycode);
                    match keycode {
                        Keycode::Num1 => self.keypad[0x1] = true,
                        Keycode::Num2 => self.keypad[0x2] = true,
                        Keycode::Num3 => self.keypad[0x3] = true,
                        Keycode::Num4 => self.keypad[0xC] = true,
                        Keycode::Q => self.keypad[0x4] = true,
                        Keycode::W => self.keypad[0x5] = true,
                        Keycode::E => self.keypad[0x6] = true,
                        Keycode::R => self.keypad[0xD] = true,
                        Keycode::A => self.keypad[0x7] = true,
                        Keycode::S => self.keypad[0x8] = true,
                        Keycode::D => self.keypad[0x9] = true,
                        Keycode::F => self.keypad[0xE] = true,
                        Keycode::Z => self.keypad[0xA] = true,
                        Keycode::X => self.keypad[0x0] = true,
                        Keycode::C => self.keypad[0xB] = true,
                        Keycode::V => self.keypad[0xF] = true,
                        _ => {}
                    }
                },  
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    info!("Key released: {:?}", keycode);
                    match keycode {
                        Keycode::Num1 => self.keypad[0x1] = false,
                        Keycode::Num2 => self.keypad[0x2] = false,
                        Keycode::Num3 => self.keypad[0x3] = false,
                        Keycode::Num4 => self.keypad[0xC] = false,
                        Keycode::Q => self.keypad[0x4] = false,
                        Keycode::W => self.keypad[0x5] = false,
                        Keycode::E => self.keypad[0x6] = false,
                        Keycode::R => self.keypad[0xD] = false,
                        Keycode::A => self.keypad[0x7] = false,
                        Keycode::S => self.keypad[0x8] = false,
                        Keycode::D => self.keypad[0x9] = false,
                        Keycode::F => self.keypad[0xE] = false,
                        Keycode::Z => self.keypad[0xA] = false,
                        Keycode::X => self.keypad[0x0] = false,
                        Keycode::C => self.keypad[0xB] = false,
                        Keycode::V => self.keypad[0xF] = false,
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }

    // Draws a pixel to the screen
    pub fn draw_pixel(&mut self, x: u8, y: u8, bit: u8) -> u8 {
        let i = (y as usize) * 64 + (x as usize);
        let prev = self.pixels[i];

        if prev == 1 && bit == 1 {
            self.shutdown_pixels[i] = 255;
        }

        self.pixels[i] ^= bit;

        prev
    }

    pub fn is_key_pressed(&self, key_value: u8) -> bool {
        self.keypad[key_value as usize]
    }

    pub fn get_key_pressed(&self) -> Option<u8> {
        for i in 0..16 {
            if self.keypad[i] {
                return Some(i as u8);
            }
        }
        None
    }

}
