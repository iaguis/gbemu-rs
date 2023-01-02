use std::{io,fs};
use std::io::Read;

pub struct Memory {
    pub rom_0: [u8; 0x3FFF+1],
    pub rom_n: [u8; 0x3FFF+1],
    ext_ram: [u8; 0x1FFF+1],
    wram_0: [u8; 0xFFF+1],
    wram_n: [u8; 0xFFF+1],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            rom_0: [0; 0x3FFF+1],
            rom_n: [0; 0x3FFF+1],
            ext_ram: [0; 0x1FFF+1],
            wram_0: [0; 0xFFF+1],
            wram_n: [0; 0xFFF+1],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0..=0x3FFF => self.rom_0[address as usize],
            0x4000..=0x7FFF => self.rom_n[address as usize - 0x3FFF],
            0xC000..=0xCFFF => self.wram_0[address as usize - 0xC000],
            0xD000..=0xDFFF => self.wram_n[address as usize - 0xD000],
            _ => panic!("bad address"),
        }
    }

    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0..=0x3FFE => self.rom_0[address as usize] = val,
            0x3FFF..=0x7FFE => self.rom_n[address as usize - 0x3FFF] = val,
            0xC000..=0xCFFF => self.wram_0[address as usize - 0xC000] = val,
            0xD000..=0xDFFF => self.wram_n[address as usize - 0xD000] = val,
            _ => panic!("bad address"),
        }
    }

    pub fn read_rom(&mut self, mut f: fs::File) -> io::Result<()> {
        f.read_exact(&mut self.rom_0)?;
        f.read_exact(&mut self.rom_n)?;

        Ok(())
    }
}
