use crate::keys::Keys;

pub struct IO {
    pub joypad: Keys,
    serial: u8,
    serial_control: u8,

    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,
}

impl IO {
    pub fn new() -> IO {
        IO {
            joypad: Keys::new(),
            serial: 0,
            serial_control: 0,
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF00 => self.joypad.read_byte(),
            0xFF01 => self.serial,
            0xFF02 => self.serial_control,
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            0xFF10..=0xFF26 => { 0 /* TODO: audio */ },
            0xFF27..=0xFF2F => { 0xFF /* TODO: audio */ },
            0xFF30..=0xFF3F => { 0 /* TODO: audio */ },
            0xFF4C..=0xFF4E => { 0 /* ??? */ },
            _ => { 0 /* TODO */ },
        }
    }

    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0xFF00 => {
                self.joypad.write_byte(val);
            },
            0xFF01 => {
                self.serial = val;
            },
            0xFF02 => {
                print!("{}", self.serial as char);
                self.serial_control = 0;
            },
            0xFF04 => { self.div = val },
            0xFF05 => { self.tima = val },
            0xFF06 => { self.tma = val },
            0xFF07 => { self.tac = val },
            0xFF10..=0xFF26 => { /* TODO: audio */ },
            0xFF27..=0xFF2F => { },
            0xFF30..=0xFF3F => { /* TODO: audio */ },
            0xFF4C..=0xFF4E => { /* ??? */ },
            _ => { /* TODO */ },
        }
    }
}
