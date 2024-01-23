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
            //Execute one CPU cycle
            cpu.step(Some(&mut screen));

            //Update the screen
            screen.update();
        }
    }
}   
