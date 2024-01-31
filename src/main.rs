mod cpu;
mod screen;

use crate::cpu::Cpu;
use crate::screen::Screen;
use log::error;
use clap::Parser;
use std::time::{Duration, Instant};

const FRAME_RATE: u16 = 40;

// Simple rust CHIP-8 interpreter
#[derive(Parser)]
struct Opts {
    // The path to the ROM file to load into memory
    rom: String,

    // The number of instructions to execute per second
    #[clap(short, long, default_value = "500")]
    ips: u16
}

fn main() {

    env_logger::init();

    let args = Opts::parse();

    let mut cpu = Cpu::new();

    if let Err(e) = cpu.load_rom_file(&args.rom) {
        error!("{:?}", e);
    } else {
        let mut screen = Screen::new();

        // Instructions per frame
        let ipf = args.ips / FRAME_RATE;

        loop {

            let start_frame = Instant::now();

            let mut draw = false;
            for _ in 0..ipf {
                cpu.step(Some(&mut screen));
                draw = draw || cpu.has_drawn();
            }
            cpu.update_timers();

            screen.update(draw);
            
            let frame_time = Instant::now().duration_since(start_frame);
            if frame_time < Duration::from_millis(1000 / FRAME_RATE as u64) {
                std::thread::sleep(Duration::from_millis(1000 / FRAME_RATE as u64) - frame_time);
            }
        }
    }
}   
