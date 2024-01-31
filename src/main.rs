mod cpu;
mod screen;

use crate::cpu::Cpu;
use crate::screen::Screen;
use log::error;
use std::env;

fn main() {

    env_logger::init();

    let mut rom_path = "roms/ibm.ch8".to_string();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        rom_path = args[1].clone();
    }

    let mut screen = Screen::new();

    let mut cpu = Cpu::new();

    if let Err(e) = cpu.load_rom_file(&rom_path) {
        error!("{:?}", e);
    } else {
        loop {
            let mut draw = false;
            for _ in 0..20 {
                cpu.step(Some(&mut screen));
                draw = draw || cpu.has_drawn();
            }
            cpu.update_timers();

            //Update the screen
            screen.update(draw);
        }
    }
}   
