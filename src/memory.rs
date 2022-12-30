use std::{io,fs};
use std::io::Read;

pub struct Memory {
    pub rom_0: [u8; 0x3FFF],
    pub rom_n: [u8; 0x3FFF],
    ext_ram: [u8; 0x1FFF],
    wram_0: [u8; 0xFFF],
    wram_n: [u8; 0xFFF],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            rom_0: [0; 0x3FFF],
            rom_n: [0; 0x3FFF],
            ext_ram: [0; 0x1FFF],
            wram_0: [0; 0xFFF],
            wram_n: [0; 0xFFF],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0..=0x3FFE => self.rom_0[address as usize],
            0x3FFF..=0x7FFE => self.rom_n[address as usize - 0x3FFF],
            0xC000..=0xCFFF => self.wram_0[address as usize - 0xC000],
            0xD000..=0xDFFF => self.wram_n[address as usize - 0xD000 - 1],
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
