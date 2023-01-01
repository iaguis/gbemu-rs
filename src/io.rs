pub struct IO {
    joypad: u8,
    serial: u8,
    serial_control: u8,

    div: u8,
    tima: u8,
    tma: u8,
    tac: u8
}

impl IO {
    pub fn new() -> IO {
        IO {
            joypad: 0,
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
            0xFF00 => self.joypad,
            0xFF01 => self.serial,
            0xFF02 => self.serial_control,
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => panic!("not implemented"),
        }
    }

    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0xFF00 => { self.joypad = val },
            0xFF01 => {
                self.serial = val;
                print!("{}", val as char);
            },
            0xFF02 => { self.serial_control = val },
            0xFF04 => { self.div = val },
            0xFF05 => { self.tima = val },
            0xFF06 => { self.tma = val },
            0xFF07 => { self.tac = val },
            _ => panic!("not implemented"),
        }
    }
}
