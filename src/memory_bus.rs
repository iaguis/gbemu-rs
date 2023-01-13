use std::{fs, io};

use crate::gpu::GPU;
use crate::memory::Memory;
use crate::keys::Keys;

pub struct MemoryBus {
    memory: Memory,
    pub joypad: Keys,
    serial: u8,
    serial_control: u8,
    pub gpu: GPU,
    pub dma: u8,
    pub interrupt_enable: Interrupts,
    pub interrupt_flag: Interrupts,
}

#[derive(Clone,Copy)]
pub struct Interrupts {
    pub vblank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

impl Interrupts {
    pub fn new() -> Interrupts {
        Interrupts {
            vblank: false,
            lcd_stat: false,
            timer: false,
            serial: false,
            joypad: false,
        }
    }
}

impl From<u8> for Interrupts {
    fn from(val: u8) -> Interrupts {
        Interrupts {
            vblank: val & 0x1 != 0,
            lcd_stat: val & 0x2 != 0,
            timer: val & 0x4 != 0,
            serial: val & 0x8 != 0,
            joypad: val & 0x10 != 0,
        }
    }
}

impl From<Interrupts> for u8 {
    fn from(val: Interrupts) -> u8 {
        let joypad = if val.joypad { 1 } else { 0 };
        let lcd_stat = if val.lcd_stat { 1 } else { 0 };
        let timer = if val.timer { 1 } else { 0 };
        let serial = if val.serial { 1 } else { 0 };
        let vblank = if val.vblank { 1 } else { 0 };
        joypad << 4 |
            lcd_stat << 3 |
            timer << 2 |
            serial << 1 |
            vblank
    }
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: Memory::new(),
            gpu: GPU::new(),
            dma: 0,
            joypad: Keys::new(),
            serial: 0,
            serial_control: 0,
            interrupt_enable: Interrupts::new(),
            interrupt_flag: Interrupts::new(),
        }
    }

    pub fn read_rom(&mut self, f: fs::File) -> io::Result<()> {
        self.memory.read_rom(f)
    }

    pub fn read_boot_rom(&mut self, f: fs::File) -> io::Result<()> {
        self.memory.read_boot_rom(f)
    }

    fn dma_transfer(&mut self) {
        let source: u16 = ((self.dma as u16) << 8) & 0xDF00;

        for obj in 0..0x9F {
            let obj_address = source + obj as u16;
            self.gpu.oam[obj] = self.read_byte(obj_address);
            self.write_byte(0xFE00 + obj as u16, self.gpu.oam[obj]);
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0..=0x7FFF => self.memory.read_byte(address),
            0x8000..=0x9FFF => self.gpu.read_byte(address),
            0xA000..=0xDFFF => self.memory.read_byte(address),
            0xE000..=0xFDFF => self.memory.read_byte(address - 0x2000),
            0xFE00..=0xFE9F => { self.gpu.read_byte(address) },
            0xFEA0..=0xFEFF => { 0 /* Not Usable */ },
            0xFF00 => self.joypad.read_byte(),
            0xFF01 => self.serial,
            0xFF02 => self.serial_control,
            0xFF03..=0xFF0E => { 0 /* ??? */ },
            0xFF0F => { self.interrupt_flag.into() },
            0xFF10..=0xFF26 => { 0 /* TODO: audio */ },
            0xFF27..=0xFF2F => { 0xFF /* TODO: audio */ },
            0xFF30..=0xFF3F => { 0 /* TODO: audio */ },
            0xFF4C..=0xFF4E => { 0 /* ??? */ },
            0xFF40..=0xFF45 => self.gpu.read_byte(address),
            0xFF46 => self.dma,
            0xFF47..=0xFF4F => self.gpu.read_byte(address),
            0xFF51..=0xFF7F => { 0 /* ??? */ },
            0xFF50 => {
                if self.memory.expose_boot_rom {
                    0
                } else {
                    1
                }
            }
            0xFF80..=0xFFFE => self.memory.read_byte(address),
            0xFFFF => { self.interrupt_enable.into() },
        }
    }

    // TODO return errors?
    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0..=0x7FFF => self.memory.write_byte(address, val),
            0x8000..=0x9FFF => self.gpu.write_byte(address, val),
            0xA000..=0xDFFF => self.memory.write_byte(address, val),
            0xE000..=0xFDFF => { },
            0xFE00..=0xFE9F => self.gpu.write_byte(address, val),
            0xFEA0..=0xFEFF => { /* Not Usable */ },
            0xFF00 => self.joypad.write_byte(val),
            0xFF01 => self.serial = val,
            0xFF02 => {
                print!("{}", self.serial as char);
                self.serial_control = 0;
            },
            0xFF03..=0xFF0E => { /* ??? */ },
            0xFF0F => { self.interrupt_flag = val.into() },
            0xFF10..=0xFF26 => { /* TODO: audio */ },
            0xFF27..=0xFF2F => { },
            0xFF30..=0xFF3F => { /* TODO: audio */ },
            0xFF4C..=0xFF4E => { /* ??? */ },
            0xFF40..=0xFF45 => self.gpu.write_byte(address, val),
            0xFF46 => {
                self.dma = val;
                self.dma_transfer();
            },
            0xFF47..=0xFF4F => self.gpu.write_byte(address, val),
            0xFF50 => {
                if val != 0 && self.memory.expose_boot_rom {
                    self.memory.expose_boot_rom = false;
                }
            }
            0xFF51..=0xFF7F => { /* ??? */ },
            0xFF80..=0xFFFE => self.memory.write_byte(address, val),
            0xFFFF => { self.interrupt_enable = val.into() },
        }
    }
}
