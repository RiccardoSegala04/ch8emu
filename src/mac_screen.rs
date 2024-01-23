use macroquad::prelude::*;
use crate::screen::miniquad::window::set_window_size;

const BLOCK_SIZE: f32 = 10.0;

// Represents the CHIP-8 screen
pub struct Screen {
    pub pixels: Vec<u8>,
}

impl Screen  {
    // Creates a new CHIP-8 screen with default values
    pub fn new() -> Screen {
        set_window_size((BLOCK_SIZE*66.0) as u32, (BLOCK_SIZE*40.0) as u32);
        Screen {
            pixels: vec![0; 64 * 32],
        }
    }

    // Clears the screen
    pub fn clear(&mut self) {
        self.pixels.iter_mut().for_each(|x| *x=0);
    }

    // Draws pixel buffer to the screen
    pub async fn update(&mut self) {

        clear_background(BLACK);

        for y in 0..32 {
            for x in 0..64 {
                let i = y * 64 + x;
                if self.pixels[i] != 1 && self.pixels[i] != 0 {
                    error!("Invalid pixel value: {}", self.pixels[i]);
                }
                if self.pixels[i] == 1 {
                    draw_rectangle(
                        (x as f32 + 1.0) * BLOCK_SIZE, 
                        (y as f32 + 1.0) * BLOCK_SIZE, 
                        BLOCK_SIZE, BLOCK_SIZE, 
                        WHITE
                    );
                }
            }
        }

        next_frame().await;
    }

    pub fn draw_pixel(&mut self, x: u8, y: u8, bit: u8) -> u8 {
        let i = (y as usize) * 64 + (x as usize);
        let prev = self.pixels[i];

        self.pixels[i] ^= bit;

        prev
    }

}
