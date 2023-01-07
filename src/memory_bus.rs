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
            0xA000..=0xDFFF => self.memory.read_byte(address),
            0xE000..=0xFDFF => self.memory.read_byte(address - 0x2000),
            0xFE00..=0xFE9F => { 0 /* TODO OAM */ },
            0xFEA0..=0xFEFF => { 0 /* Not Usable */ },
            0xFF00..=0xFF3F | 0xFF51..=0xFF7F => self.io.read_byte(address),
            0xFF40..=0xFF4F => self.gpu.read_byte(address),
            0xFF50 => {
                if self.memory.expose_boot_rom {
                    0
                } else {
                    1
                }
            }
            0xFF80..=0xFFFE => self.memory.read_byte(address),
            0xFFFF => { 0 /* TODO Interrupt flag */ },
        }
    }

    // TODO return errors?
    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0..=0x7FFF => self.memory.write_byte(address, val),
            0x8000..=0x9FFF => self.gpu.write_byte(address, val),
            0xA000..=0xDFFF => self.memory.write_byte(address, val),
            0xE000..=0xFDFF => { },
            0xFE00..=0xFE9F => { /* TODO OAM */ },
            0xFEA0..=0xFEFF => { /* Not Usable */ },
            0xFF00..=0xFF3F | 0xFF51..=0xFF7F => self.io.write_byte(address, val),
            0xFF40..=0xFF4F => self.gpu.write_byte(address, val),
            0xFF50 => {
                if val != 0 && self.memory.expose_boot_rom {
                    self.memory.expose_boot_rom = false;
                }
            }
            0xFF80..=0xFFFE => self.memory.write_byte(address, val),
            0xFFFF => { /* TODO Interrupt flag */ },
        }
    }
}
