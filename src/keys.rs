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

    pub fn key_down(&mut self, k: &winit::event::VirtualKeyCode) {
        match k {
            winit::event::VirtualKeyCode::Up => self.rows[1] &= 0xB,
            winit::event::VirtualKeyCode::Down => self.rows[1] &= 0x7,
            winit::event::VirtualKeyCode::Left => self.rows[1] &= 0xD,
            winit::event::VirtualKeyCode::Right => self.rows[1] &= 0xE,
            // B
            winit::event::VirtualKeyCode::A => self.rows[0] &= 0xD,
            // A
            winit::event::VirtualKeyCode::S => {
                self.rows[0] &= 0xE;
            },
            // Start
            winit::event::VirtualKeyCode::G => self.rows[0] &= 0x7,
            // Select
            winit::event::VirtualKeyCode::H => self.rows[0] &= 0xB,
            _ => {},
        }
    }

    pub fn key_up(&mut self, k: &winit::event::VirtualKeyCode) {
        match k {
            winit::event::VirtualKeyCode::Up => self.rows[1] |= 0x4,
            winit::event::VirtualKeyCode::Down => self.rows[1] |= 0x8,
            winit::event::VirtualKeyCode::Left => self.rows[1] |= 0x2,
            winit::event::VirtualKeyCode::Right => self.rows[1] |= 0x1,
            // B
            winit::event::VirtualKeyCode::A => self.rows[0] |= 0x2,
            // A
            winit::event::VirtualKeyCode::S => self.rows[0] |= 0x1,
            // Start
            winit::event::VirtualKeyCode::G => self.rows[0] |= 0x8,
            // Select
            winit::event::VirtualKeyCode::H => self.rows[0] |= 0x4,
            _ => {},
        }
    }
}

