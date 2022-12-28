use std::{io,fs};

use crate::memory::Memory;
use crate::gpu::GPU;

pub struct MemoryBus {
    memory: Memory,
    gpu: GPU,
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: Memory::new(),
            gpu: GPU::new(),
        }
    }

    pub fn read_rom(&mut self, mut f: fs::File) -> io::Result<()> {
        self.memory.read_rom(f)
    }

    pub fn read_byte(&self, address: usize) -> u8 {
        match address {
            0..=0x7FFE => self.memory.read_byte(address),
            0x8000..=0x9FFF => self.gpu.read_byte(address),
            // FIXME
            _ => panic!("bad address"),
        }
    }

    // TODO return errors?
    pub fn write_byte(&mut self, address: usize, val: u8) {
        match address {
            0..=0x7FFE => self.memory.write_byte(address, val),
            0x8000..=0x9FFF => self.gpu.write_byte(address, val),
            _ => panic!("bad address"),
        }
    }
}
