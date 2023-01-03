use std::{fs, io};

use crate::gpu::GPU;
use crate::io::IO;
use crate::memory::Memory;

pub struct MemoryBus {
    memory: Memory,
    io: IO,
    pub gpu: GPU,
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: Memory::new(),
            gpu: GPU::new(),
            io: IO::new(),
        }
    }

    pub fn read_rom(&mut self, f: fs::File) -> io::Result<()> {
        self.memory.read_rom(f)
    }

    pub fn read_boot_rom(&mut self, f: fs::File) -> io::Result<()> {
        self.memory.read_boot_rom(f)
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0..=0x7FFF => self.memory.read_byte(address),
            0x8000..=0x9FFF => self.gpu.read_byte(address),
            0xC000..=0xDFFF => self.memory.read_byte(address),
            0xFF00..=0xFF4F | 0xFF51..=0xFF7F => self.io.read_byte(address),
            0xFF50 => {
                if self.memory.expose_boot_rom {
                    0
                } else {
                    1
                }
            }
            // FIXME
            _ => {
                print!("reading {:#04x}: ", address);
                panic!("bad address");
            }
        }
    }

    // TODO return errors?
    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0..=0x7FFE => self.memory.write_byte(address, val),
            0x8000..=0x9FFF => self.gpu.write_byte(address, val),
            0xC000..=0xDFFF => self.memory.write_byte(address, val),
            0xFF00..=0xFF4F | 0xFF51..=0xFF7F => self.io.write_byte(address, val),
            0xFF50 => {
                if val != 0 && self.memory.expose_boot_rom {
                    self.memory.expose_boot_rom = false;
                }
            }
            _ => {
                print!("writing {:#04x}: ", address);
                panic!("bad address");
            }
        }
    }
}
