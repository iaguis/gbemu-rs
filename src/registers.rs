pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,

    pub sp: u16,
    pub pc: u16,

    pub m: u8,
    pub ticks: u32,
}

pub enum Flag {
    C = 4,
    H,
    N,
    Z,
}

impl Registers {
    pub fn new() -> Registers {
        // https://raw.githubusercontent.com/AntonioND/giibiiadvance/master/docs/TCAGBD.pdf page 10
        Registers {
            a: 0x01,
            b: 0,
            c: 0x13,
            d: 0,
            e: 0xd8,
            f: 0xb0,
            h: 0x1,
            l: 0x4d,
            sp: 0xfffe,
            pc: 0x100,
            m: 0,
            ticks: 0,
        }
    }

    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        return (self.f & (0b00000001 << (flag as u8))) != 0;
    }

    pub fn set_flag(&mut self, flag: Flag, val: bool) {
        if val {
            self.f |= (0b00000001 << (flag as u8))
        } else {
            self.f &= !(0b00000001 << (flag as u8))
        }
    }

    pub fn alu_add(&mut self, b: u8) {
        let a = self.a;
        let c = self.get_flag(Flag::C) as u8;
        let r = a.wrapping_add(b).wrapping_add(c);

        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, false);
        // half carry
        self.set_flag(Flag::H, (a & 0xF) + (b & 0xF) + c > 0xF);
        self.set_flag(Flag::C, (a as u16) + (b as u16) + (c as u16) > 0xFF);
        self.a = r;
    }

    fn alu_inc(&mut self, a: u8) -> u8 {
        let r = a.wrapping_add(1);
        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, (a & 0xf) + 1 > 0xF);

        r
    }

    fn alu_dec(&mut self, a: u8) -> u8 {
        let r = a.wrapping_sub(1);
        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, (a & 0xf) == 0);

        r
    }

    // FIXME generalize to other registers?
    pub fn inc_b(&mut self) {
        self.b = self.alu_inc(self.b);
    }

    pub fn dec_b(&mut self) {
        self.b = self.alu_dec(self.b);
    }

    pub fn inc_bc(&mut self) {
        // FIXME this is wrong
        self.c += 1;
        if self.c == 0 {
            self.b += 1;
        }

        self.set_flag(Flag::Z, false);
        if self.bc() == 0 {
            self.set_flag(Flag::Z, true);
        }

        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        // TODO if carry_per_bit[3] set_flag(Flag::H, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_a_and_f() {
        let mut mock_registers = Registers::new();

        mock_registers.a = 0x10;
        mock_registers.f = 0xff;

        assert_eq!(mock_registers.af(), 0x10ff);
    }
}
