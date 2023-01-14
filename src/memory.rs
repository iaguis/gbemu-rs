use std::{io,fs};
use std::io::Read;

pub struct Memory {
    pub boot_rom: [u8; 0x100],
    pub rom: Vec<u8>,
    ext_ram: [u8; 0x1FFF+1],
    wram_0: [u8; 0xFFF+1],
    wram_n: [u8; 0xFFF+1],
    pub hram: [u8; 0x7E+1],
    pub expose_boot_rom: bool,

    // MBC
    cartridge_type: u8,
    rom_offset: u16,
    ram_offset: u16,
    mbc_internal: MBC,
}

pub struct MBC {
    rom_bank: u8,
    ram_bank: u8,
    enable_ext_ram: bool,
    mode: MBCMode
}

pub enum MBCMode {
    ROM,
    RAM,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            boot_rom: [0; 0x100],
            rom: Vec::new(),
            ext_ram: [0; 0x1FFF+1],
            wram_0: [0; 0xFFF+1],
            wram_n: [0; 0xFFF+1],
            hram: [0; 0x7E+1],
            expose_boot_rom: false,
            cartridge_type: 0,
            rom_offset: 0x4000,
            ram_offset: 0,
            mbc_internal: MBC {
                rom_bank: 0,
                ram_bank: 0,
                enable_ext_ram: false,
                mode: MBCMode::ROM,
            }
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0..=0xff => { if self.expose_boot_rom { self.boot_rom[address as usize] } else { self.rom[address as usize] } },
            0x100..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[(self.rom_offset + (address & 0x3FFF)) as usize],
            0xA000..=0xBFFF => self.ext_ram[(self.ram_offset + (address & 0x1FFF)) as usize],
            0xC000..=0xCFFF => self.wram_0[address as usize - 0xC000],
            0xD000..=0xDFFF => self.wram_n[address as usize - 0xD000],
            0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80],
            _ => { panic!("bad address: {:#04x}", address); },
        }
    }

    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0..=0x1FFF => {
                match self.cartridge_type {
                    0x02..=0x03 => {
                        self.mbc_internal.enable_ext_ram = (val & 0x0F) == 0x0A;
                    },
                    _ => {},
                }
            }
            0x2000..=0x3FFF => {
                match self.cartridge_type {
                    0x01..=0x03 => {
                        let mut val = val & 0x1F;
                        if val == 0 {
                            val = 1;
                        }
                        self.mbc_internal.rom_bank = (self.mbc_internal.rom_bank & 0x60) + val;

                        self.rom_offset = self.mbc_internal.rom_bank as u16 * 0x4000;
                    },
                    _ => {},
                }
            },
            0x4000..=0x5FFF => {
               match self.cartridge_type {
                    0x01..=0x03 => {
                        match self.mbc_internal.mode {
                            MBCMode::RAM => {
                                self.mbc_internal.ram_bank = val & 0x3;
                                self.ram_offset = self.mbc_internal.ram_bank as u16 * 0x2000;
                            },
                            MBCMode::ROM => {
                                self.mbc_internal.rom_bank = (self.mbc_internal.rom_bank & 0x1F) + ((val & 0x3) << 5);
                            },
                        }
                        let mut val = val & 0x1F;
                        if val == 0 {
                            val = 1;
                        }
                        self.mbc_internal.rom_bank = (self.mbc_internal.rom_bank & 0x60) + val;

                        self.rom_offset = self.mbc_internal.rom_bank as u16 * 0x4000;
                    },
                    _ => {},
               }
            },
            0x6000..=0x7FFF => {
                match self.cartridge_type {
                    0x02..=0x03 => self.mbc_internal.mode = if (val & 0x1) == 1 { MBCMode::RAM } else { MBCMode::ROM },
                    _ => {},
                }
            }
            0xA000..=0xBFFF => self.ext_ram[address as usize - 0xA000] = val,
            0xC000..=0xCFFF => self.wram_0[address as usize - 0xC000] = val,
            0xD000..=0xDFFF => self.wram_n[address as usize - 0xD000] = val,
            0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80] = val,
            _ => { panic!("bad address: {:#04x}", address); },
        }
    }

    pub fn read_rom(&mut self, rom_path: &str) -> io::Result<()> {
        self.rom = fs::read(rom_path).expect("can't read ROM");

        self.cartridge_type = self.rom[0x0147];

        Ok(())
    }

    pub fn read_boot_rom(&mut self, mut f: fs::File) -> io::Result<()> {
        f.read_exact(&mut self.boot_rom)
    }
}
