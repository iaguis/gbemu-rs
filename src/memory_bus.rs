use std::{fs, io};

use crate::gpu::GPU;
use crate::memory::Memory;
use crate::keys::Keys;

pub struct InternalClock {
    main: u32,
    sub: u32,
    div: u32,
}

pub struct Clock {
    internal: InternalClock,

    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: Tac,
}

#[derive(Clone,Copy)]
pub enum ClockSelect {
    Freq4k,
    Freq256k,
    Freq64k,
    Freq16k,
}

#[derive(Clone,Copy)]
pub struct Tac {
    enable: bool,
    clock_select: ClockSelect,
}

impl Tac {
    pub fn new() -> Tac {
        Tac {
            enable: false,
            clock_select: ClockSelect::Freq4k,
        }
    }
}

impl From<u8> for Tac {
    fn from(value: u8) -> Tac {
        Tac {
            enable: (value >> 2 & 0x01) == 1,
            clock_select: match value & 0x3 {
                0 => ClockSelect::Freq4k,
                1 => ClockSelect::Freq256k,
                2 => ClockSelect::Freq64k,
                3 => ClockSelect::Freq16k,
                _ => panic!("wrong frequency"),
            },
        }
    }
}

impl From<Tac> for u8 {
    fn from(value: Tac) -> u8 {
        let enable = if value.enable {1} else {0};
        let clock_select = match value.clock_select {
            ClockSelect::Freq4k => 0,
            ClockSelect::Freq256k => 1,
            ClockSelect::Freq64k => 2,
            ClockSelect::Freq16k => 3,
        };

        enable << 2 | clock_select
    }
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            internal: InternalClock{
                main: 0,
                sub: 0,
                div: 0,
            },
            div: 0,
            tima: 0,
            tma: 0,
            tac: Tac::new(),
        }
    }

    pub fn inc(&mut self, cycles: u32) -> bool {
        self.internal.sub += cycles;

        // overflow
        if self.internal.sub >= 4 {
            self.internal.main += 1;
            self.internal.sub -= 4;

            self.internal.div += 1;
            if self.internal.div == 16 {
                self.div = self.div.wrapping_add(1);
                self.internal.div = 0;
            }
        }

        self.check()
    }

    pub fn check(&mut self) -> bool {
        if !self.tac.enable {
            return false
        }

        let threshold = match self.tac.clock_select {
            ClockSelect::Freq4k => 64,
            ClockSelect::Freq256k => 1,
            ClockSelect::Freq64k => 4,
            ClockSelect::Freq16k => 16,
        };

        if self.internal.main >= threshold {
            return self.step();
        }

        return false;
    }

    pub fn step(&mut self) -> bool {
        self.internal.main = 0;
        let overflow: bool;

        (self.tima, overflow) = self.tima.overflowing_add(1);

        if overflow {
            self.tima = self.tma;

            return true;
        }
        return false;
    }
}

pub struct MemoryBus {
    pub memory: Memory,
    pub joypad: Keys,
    serial: u8,
    serial_control: u8,
    pub clock: Clock,
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
            clock: Clock::new(),
        }
    }

    pub fn read_rom(&mut self, rom_path: &str) -> io::Result<()> {
        self.memory.read_rom(rom_path)
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
            0xFF03 => { 0 /* ??? */ },
            // TODO fix types?
            0xFF04 => { self.clock.div },
            0xFF05 => { self.clock.tima },
            0xFF06 => { self.clock.tma },
            0xFF07 => { self.clock.tac.into() },
            0xFF08..=0xFF0E => { 0 /* ??? */ },
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
            0xFF03 => { /* ??? */ },
            0xFF04 => { self.clock.div = 0; },
            0xFF05 => { self.clock.tima = val; },
            0xFF06 => { self.clock.tma = val; },
            0xFF07 => { self.clock.tac = val.into(); },
            0xFF08..=0xFF0E => { /* ??? */ },
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
