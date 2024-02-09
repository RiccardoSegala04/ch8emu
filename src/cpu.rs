use std::fs::File;
use std::io::{self, Read};
use log::{info, warn, trace};
use crate::screen::Screen;
use rand::Rng;
use std::time::{Duration, Instant};

// Memory address where CHIP-8 programs usually start
const START_PGM: u16 = 0x200;

// Memory address where the fontset starts
const START_FONT: u16 = 0x50;

// Size of the CHIP-8 RAM in bytes
const RAM_SIZE: usize = 4096;

// Represents the state of the CHIP-8 CPU
pub struct Cpu {
    pc: u16,
    sp: u16,

    index: u16,
    v_reg: [u8; 16],

    delay_timer: u8,
    sound_timer: u8,

    ram: [u8; RAM_SIZE],
    
    time: Instant,

    last_key: Option<u8>,

    has_drawn: bool,
}


impl Cpu {

    // Creates and initializes a new CHIP-8 CPU instance with default values
    pub fn new() -> Cpu {
        Cpu {
            pc: START_PGM,
            sp: 0,
            index: 0,
            v_reg: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            ram: [0; 4096],
            time: Instant::now(),
            last_key: None,
            has_drawn: false,
        }
    }

    // Loads a ROM into the CPU's memory, from the program start address
    pub fn load_rom(&mut self, rom: &[u8]) {

        let mut startcpy: usize = START_PGM as usize;
        for byte in rom {
            self.ram[startcpy] = *byte;
            startcpy+=1;
        }

        // Load the fontset into the memory
        let fontset: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F];
        ];

        startcpy = START_FONT as usize;
        for byte in fontset.iter() {
            self.ram[startcpy] = *byte;
            startcpy+=1;
        }

        info!("Loaded {} bytes from the disk", rom.len());
    }
    
    // Loads a CHIP-8 ROM from a file into the CPU's memory
    pub fn load_rom_file(&mut self, path: &str) -> io::Result<()> {
        let mut file = File::open(path)?;

        let mut buf = vec!();
        file.read_to_end(&mut buf)?;

        self.load_rom(&buf);

        Ok(())
    }

    pub fn update_timers(&mut self) {
        // Update timers
        let now = Instant::now();

        trace!("{}", now.duration_since(self.time).as_millis());

        // Update timers every 16ms (~ 60Hz)
        if now.duration_since(self.time) >= Duration::from_millis(16) {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }

            self.time = now;
        }
    }


    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }

    pub fn has_drawn(&self) -> bool {
        self.has_drawn
    }

    // Executes one step of the CHIP-8 CPU
    pub fn step(&mut self, screen: Option<&mut Screen>) {

        self.has_drawn = false;
        let opcode = self.fetch();

        trace!("Executing 0x{:x}", opcode);

        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    // Clear the screen
                    0xE0 => {
                        self.has_drawn = true;
                        let screen = screen.unwrap();
                        trace!("Clearing the screen");
                        screen.clear();
                    }

                    // Return from subroutine
                    0xEE => {
                        trace!("Returning from subroutine");
                        self.sp-=1;
                        self.pc = self.ram[self.sp as usize] as u16;
                        self.sp-=1;
                        self.pc = self.pc<<8 | (self.ram[self.sp as usize] as u16);
                    }

                    _ => warn!("Operation 0x{:x} is not implemented yet!", opcode),
                }
            },
            // Jump to address NNN
            0x1000 => {
                trace!("Jumping to 0x{:x}", opcode & 0x0FFF);
                self.pc = opcode & 0x0FFF;
            },
            // Call subroutine
            0x2000 => {
                trace!("Calling subroutine at 0x{:x}", opcode & 0x0FFF);
                self.ram[self.sp as usize] = (self.pc & 0xff) as u8;
                self.sp+=1;
                self.ram[self.sp as usize] = (self.pc>>8) as u8;
                self.sp+=1;

                self.pc = opcode & 0x0fff;
            },
            // Skip next instruction if VX == NN
            0x3000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                trace!("Skip if V{} == {}", x, nn);

                if self.v_reg[x as usize] == nn as u8 {
                    self.pc += 2;
                }
            },
            // Skip next instruction if VX != NN
            0x4000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                trace!("Skip if V{} != {}", x, nn);

                if self.v_reg[x as usize] != nn as u8 {
                    self.pc += 2;
                }
            },
            // Skip next instruction if VX == VY
            0x5000 => {
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;

                trace!("Skip if V{} == V{}", x, y);

                if self.v_reg[x as usize] == self.v_reg[y as usize] {
                    self.pc += 2;
                }
            },
            // Set VX to NN
            0x6000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                trace!("Setting V{} to {}", x, nn);
                self.v_reg[x as usize] = nn as u8;
            },
            // Add NN to VX
            0x7000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                trace!("Adding {} to V{}", nn, x);
                self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(nn as u8);
            },
            // Arithmetical logical operations
            0x8000 => {
                match opcode & 0xF {
                    // Setting VX = VY
                    0x0 => {
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;

                        trace!("Setting V{} = V{}", x, y);

                        self.v_reg[x as usize] = self.v_reg[y as usize];
                    },
                    // Setting VX |= VY
                    0x1 => {
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;

                        trace!("Setting V{} |= V{}", x, y);

                        self.v_reg[x as usize] |= self.v_reg[y as usize];
                        self.v_reg[0xf] = 0;
                    },
                    // Setting VX &= VY
                    0x2 => {
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;

                        trace!("Setting V{} &= V{}", x, y);

                        self.v_reg[x as usize] &= self.v_reg[y as usize];
                        self.v_reg[0xf] = 0;
                    },
                    // Setting VX ^= VY
                    0x3 => {
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;

                        self.v_reg[x as usize] ^= self.v_reg[y as usize];
                        self.v_reg[0xf] = 0;
                    },
                    // Add VY to VX (affects the carry flag)
                    0x4 => {
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;

                        trace!("Adding V{} to V{} (carry)", y, x);

                        let mut flag: u8 = 0;

                        if self.v_reg[x as usize] as u16 + self.v_reg[y as usize] as u16 > 255 {
                            flag = 1;
                        } 

                        self.v_reg[x as usize] = 
                            self.v_reg[x as usize]
                                .wrapping_add(self.v_reg[y as usize]);

                        self.v_reg[0xF] = flag;
                    },
                    // Subtract VY from VX (affects the carry flag)
                    0x5 => {
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;

                        let mut flag: u8 = 0;
                        if self.v_reg[x as usize] >= self.v_reg[y as usize] {
                            flag = 1;
                        }

                        self.v_reg[x as usize] = 
                            self.v_reg[x as usize]
                                .wrapping_sub(self.v_reg[y as usize]);

                        self.v_reg[0xF] = flag;
                    },
                    // Set VX = VY >> 1 (affects the carry flag)
                    0x6 => {
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;

                        let flag: u8 = self.v_reg[y as usize] & 0x01; 
                        self.v_reg[x as usize] = self.v_reg[y as usize] >> 1;
                        self.v_reg[0xF] = flag;
                    },
                    // Subtract VX from VY (affects the carry flag)
                    0x7 => {
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;

                        let mut flag: u8 = 0;
                        if self.v_reg[y as usize] >= self.v_reg[x as usize] {
                            flag = 1;
                        }

                        self.v_reg[x as usize] = 
                            self.v_reg[y as usize]
                                .wrapping_sub(self.v_reg[x as usize]);

                        self.v_reg[0xF] = flag;
                    },
                    // Set VX = VY << 1 (affects the carry flag)
                    0xE => {
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;
                        
                        let flag: u8 = (self.v_reg[y as usize] & 0x80) >> 7;
                        self.v_reg[x as usize] = self.v_reg[y as usize] << 1;
                        self.v_reg[0xF] = flag;
                    },

                    _ => warn!("Operation 0x{:x} is not implemented yet!", opcode),
                }
            },
            // Skip next instruction if VX != VY
            0x9000 => {
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;

                trace!("Skip if V{} != V{}", x, y);

                if self.v_reg[x as usize] != self.v_reg[y as usize] {
                    self.pc += 2;
                }
            },
            // Set index to NNN
            0xA000 => {
                let nnn = opcode & 0x0FFF;
                trace!("Setting index to 0x{:x}", nnn);
                self.index = nnn;
            },
            // Jump to NNN + V0
            0xB000 => {
                let nnn = opcode & 0x0FFF;
                trace!("Jumping to 0x{:x} + V0 (0x{:x})", nnn, self.v_reg[0]);
                self.pc = nnn + self.v_reg[0] as u16;
            },
            // Set VX to random number & NN
            0xC000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = (opcode & 0x00FF) as u8;
                let mut rng = rand::thread_rng();

                trace!("Setting V{} to random number & {}", x, nn);

                self.v_reg[x as usize] = rng.gen::<u8>() & nn;

            }
            // Draw sprite
            0xD000 => { 
                self.has_drawn = true;

                let screen = screen.unwrap();
                
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                let n = opcode & 0x000F;

                let x = self.v_reg[x as usize] % 64;
                let y = self.v_reg[y as usize] % 32;

                self.v_reg[0xf] = 0;
                for i in 0..n {
                    if y+(i as u8) >= 32 {
                        break;
                    }
                    let byte = self.ram[(self.index + i) as usize];
                    for j in 0..8 {
                        if x+j >= 64 {
                            break;
                        }
                        let bit = (byte >> (7-j)) & 0x01;
                        let prev = screen.draw_pixel(x+j, y+(i as u8), bit);
                        if prev == 1 && bit == 1 {
                            self.v_reg[0xf] = 1;
                        }
                    }
                    
                }
            }

            0xE000 => {
                let screen = screen.unwrap();
                match opcode & 0x00FF {
                    // Skip next instruction if key VX is pressed
                    0x9E => {
                        let x = (opcode & 0x0F00) >> 8;
                        if screen.is_key_pressed(self.v_reg[x as usize]) {
                            trace!("Key V{} is pressed", x);
                            self.pc += 2;
                        }
                    },

                    // Skip next instruction if key VX is not pressed
                    0xA1 => {
                        let x = (opcode & 0x0F00) >> 8;
                        if !screen.is_key_pressed(self.v_reg[x as usize]) {
                            trace!("Key V{} is not pressed", x);
                            self.pc += 2;
                        }
                    },

                    _ => warn!("Operation 0x{:x} is not implemented yet!", opcode),
                }
            }

            0xF000 => {
                match opcode & 0x00FF {
                    // Set VX = delay timer
                    0x07 => {
                        trace!("Setting V{} = delay timer", (opcode & 0x0F00) >> 8);
                        let x = (opcode & 0x0F00) >> 8;
                        self.v_reg[x as usize] = self.delay_timer;
                    },
                    // Set delay timer = VX
                    0x15 => {
                        trace!("Setting delay timer = V{}", (opcode & 0x0F00) >> 8);
                        let x = (opcode & 0x0F00) >> 8;
                        self.delay_timer = self.v_reg[x as usize];
                    },
                    // Set sound timer = VX
                    0x18 => {
                        trace!("Setting sound timer = V{}", (opcode & 0x0F00) >> 8);
                        let x = (opcode & 0x0F00) >> 8;
                        self.sound_timer = self.v_reg[x as usize];
                    },
                    // Set index = index + VX 
                    0x1E => {
                        let x = (opcode & 0x0F00) >> 8;
                        trace!("Setting index = index + V{}", x);

                        self.index = self.index.wrapping_add(self.v_reg[x as usize] as u16);
                    },
                    0x29 => {
                        let x = (opcode & 0x0F00) >> 8;
                        trace!("Setting index = sprite address of V{}", x);
                        self.index = START_FONT+(self.v_reg[x as usize]*5) as u16;
                    },
                    0x33 => {
                        let x = (opcode & 0x0F00) >> 8;
                        trace!("Storing BCD representation of V{} in memory", x);
                        self.ram[self.index as usize] = self.v_reg[x as usize] / 100;
                        self.ram[(self.index+1) as usize] = (self.v_reg[x as usize] / 10) % 10;
                        self.ram[(self.index+2) as usize] = self.v_reg[x as usize] % 10;
                    },
                    // Store v_reg[0]..v_reg[x] in memory starting at index
                    0x55 => {
                        let x = (opcode & 0x0F00) >> 8;
                        trace!("Storing v_reg[0]..v_reg[{}] in memory starting at index", x);
                        for i in 0..x+1 {
                            self.ram[(self.index) as usize] = self.v_reg[i as usize];
                            self.index += 1;
                        }
                    },
                    // Read v_reg[0]..v_reg[x] from memory starting at index
                    0x65 => {
                        let x = (opcode & 0x0F00) >> 8;
                        trace!("Reading v_reg[0]..v_reg[{}] from memory starting at index", x);
                        for i in 0..x+1 {
                            self.v_reg[i as usize] = self.ram[(self.index) as usize];
                            self.index += 1;
                        }
                    },

                    0x0A => {
                        let screen = screen.unwrap();
                        let x = (opcode & 0x0F00) >> 8;

                        match self.last_key {
                            Some(key) => {
                                if !screen.is_key_pressed(key) {
                                    self.v_reg[x as usize] = key;
                                    self.last_key = None;
                                } else {
                                    self.pc -= 2;
                                }
                            },
                            None => {
                                self.pc -= 2;
                                self.last_key = screen.get_key_pressed();
                            }
                        }

                    }

                    _ => warn!("Operation 0x{:x} is not implemented yet!", opcode),
                }
            }
            
            _ => warn!("Operation {opcode} is not implemented yet!"),
        }

    }

    // Fetches the next opcode from the memory and advances the program counter
    fn fetch(&mut self) -> u16 {
        let mut opcode: u16 = self.ram[self.pc as usize].into();
        opcode = opcode << 8 | self.ram[(self.pc+1) as usize] as u16;
        self.pc += 2;

        opcode
    }

    
}

#[cfg(test)]
mod test {
    #[test]
    fn jump() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x10,0x01]);
        cpu.step(None);
        assert_eq!(cpu.pc, 0x0001);
    }
    
    #[test]
    fn set_vx() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60,0x01]);
        cpu.step(None);
        assert_eq!(cpu.v_reg[0], 0x01);
    }
    
    #[test]
    fn add_vx() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x70, 0x01]);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.v_reg[0], 0x02);
    }

    #[test]
    fn set_index() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0xA0, 0x01]);
        cpu.step(None);
        assert_eq!(cpu.index, 0x0001);
    }

    #[test]
    fn call_sub() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x20, 0x01]);
        cpu.step(None);
        assert_eq!(cpu.pc, 0x0001);
        assert_eq!(cpu.sp, 0x0002);
        assert_eq!(cpu.ram[0x0000], 0x02);
        assert_eq!(cpu.ram[0x0001], 0x02);
    }

    #[test]
    fn ret_sub() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x22, 0x02, 0x00, 0xEE]);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.sp, 0x0000);
    }

    #[test]
    fn skip_vx_eq_nn() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x30, 0x01]);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn skip_vx_neq_nn() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x40, 0x02]);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn skip_vx_eq_vy() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x61, 0x01, 0x50, 0x10]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.pc, 0x208);
    }

    #[test]
    fn skip_vx_neq_vy() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x61, 0x02, 0x90, 0x10]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.pc, 0x208);
    }

    #[test]
    fn set_vx_vy() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x61, 0x02, 0x80, 0x10]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.v_reg[0], 0x02);
    }

    #[test]
    fn set_vx_vx_or_vy() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x61, 0x02, 0x80, 0x11]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.v_reg[0], 0x03);
    }

    #[test]
    fn set_vx_vx_and_vy() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x61, 0x02, 0x80, 0x12]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.v_reg[0], 0x00);
    }

    #[test]
    fn set_vx_vx_xor_vy() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x61, 0x02, 0x80, 0x13]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);
        assert_eq!(cpu.v_reg[0], 0x03);
    }

    #[test]
    fn add_vx_vy_carry() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0x61, 0xFF, 0x80, 0x14]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);

        assert_eq!(cpu.v_reg[0], 0x00);
        assert_eq!(cpu.v_reg[0xF], 0x01);
    }

    #[test]
    fn sub_vx_vy_carry() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x61, 0xFF, 0x60, 0x01, 0x80, 0x15]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);

        assert_eq!(cpu.v_reg[0], 0x02);
        assert_eq!(cpu.v_reg[0xF], 0x00);
    }

    #[test]
    fn sub_vy_vx_carry() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0xFF, 0x61, 0x01, 0x80, 0x17]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);

        assert_eq!(cpu.v_reg[0], 0x02);
        assert_eq!(cpu.v_reg[0xF], 0x00);
    }

    #[test]
    fn set_vx_vy_shr() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x61, 0x03, 0x80, 0x16]);
        cpu.step(None);
        cpu.step(None);

        assert_eq!(cpu.v_reg[0], 0x01);
        assert_eq!(cpu.v_reg[0xF], 0x01);
    }

    #[test]
    fn set_vx_vy_shl() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x61, 0x80, 0x80, 0x1E]);
        cpu.step(None);
        cpu.step(None);

        assert_eq!(cpu.v_reg[0], 0x00);
        assert_eq!(cpu.v_reg[0xF], 0x01);
    }

    #[test]
    fn jump_with_offset() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0xB0, 0x01]);
        cpu.step(None);
        cpu.step(None);

        assert_eq!(cpu.pc, 0x0002);
    }
    
    #[test]
    fn add_idx_vx() {
        let mut cpu = super::Cpu::new();
        cpu.load_rom(&[0x60, 0x01, 0xA0, 0x01, 0xF0, 0x1E]);
        cpu.step(None);
        cpu.step(None);
        cpu.step(None);

        assert_eq!(cpu.index, 0x0002);
        assert_eq!(cpu.v_reg[0xF], 0x00);
    }

}
