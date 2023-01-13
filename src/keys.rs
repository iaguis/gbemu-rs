pub struct Keys {
    rows: [u8; 2],
    column: u8
}

impl Keys {
    pub fn new() -> Keys {
        Keys {
            rows: [0x0F; 2],
            column: 0,
        }
    }

    pub fn read_byte(&self) -> u8 {
        match self.column {
            0x10 => {
                self.rows[0]
            },
            0x20 => self.rows[1],
            _ => { 0 },
        }
    }

    pub fn write_byte(&mut self, val: u8) {
        self.column = val & 0x30;
    }

    pub fn key_down(&mut self, k: &minifb::Key) {
        match k {
            minifb::Key::Up => self.rows[1] &= 0xB,
            minifb::Key::Down => self.rows[1] &= 0x7,
            minifb::Key::Left => self.rows[1] &= 0xD,
            minifb::Key::Right => self.rows[1] &= 0xE,
            // B
            minifb::Key::A => self.rows[0] &= 0xD,
            // A
            minifb::Key::S => {
                self.rows[0] &= 0xE;
            },
            // Start
            minifb::Key::G => self.rows[0] &= 0x7,
            // Select
            minifb::Key::H => self.rows[0] &= 0xB,
            _ => {},
        }
    }

    pub fn key_up(&mut self, k: &minifb::Key) {
        match k {
            minifb::Key::Up => self.rows[1] |= 0x4,
            minifb::Key::Down => self.rows[1] |= 0x8,
            minifb::Key::Left => self.rows[1] |= 0x2,
            minifb::Key::Right => self.rows[1] |= 0x1,
            // B
            minifb::Key::A => self.rows[0] |= 0x2,
            // A
            minifb::Key::S => self.rows[0] |= 0x1,
            // Start
            minifb::Key::G => self.rows[0] |= 0x8,
            // Select
            minifb::Key::H => self.rows[0] |= 0x4,
            _ => {},
        }
    }
}

