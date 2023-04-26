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
}

pub enum Flag {
    C = 4,
    H,
    N,
    Z,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0x100,
        }
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = ((val & 0xff00) >> 8) as u8;
        self.c = (val & 0xff) as u8;
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = ((val & 0xff00) >> 8) as u8;
        self.e = (val & 0xff) as u8;
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = ((val & 0xff00) >> 8) as u8;
        self.l = (val & 0xff) as u8;
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        return (self.f & (0b00000001 << (flag as u8))) != 0;
    }

    pub fn set_flag(&mut self, flag: Flag, val: bool) {
        if val {
            self.f |= 0b00000001 << (flag as u8)
        } else {
            self.f &= !(0b00000001 << (flag as u8))
        }
    }

    fn alu_add_internal(&mut self, b: u8, with_carry: bool) {
        let a = self.a;
        let c = if self.get_flag(Flag::C) { 1 } else { 0 };
        let mut r = a.wrapping_add(b);

        if with_carry {
            r = r.wrapping_add(c);
        }

        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, false);

        if with_carry {
            self.set_flag(Flag::C, (a as u16) + (b as u16) + (c as u16) > 0xFF);
            self.set_flag(Flag::H, (a & 0xF) + (b & 0xF) + c > 0xF);
        } else {
            self.set_flag(Flag::C, (a as u16) + (b as u16) > 0xFF);
            self.set_flag(Flag::H, (a & 0xF) + (b & 0xF) > 0xF);
        }

        self.a = r;
    }

    pub fn alu_add(&mut self, b: u8) {
        self.alu_add_internal(b, false)
    }

    pub fn alu_addhl(&mut self, b: u16) {
        let hl = self.hl();
        let (r, carry) = hl.overflowing_add(b);

        self.set_flag(Flag::N, false);
        self.set_flag(Flag::C, carry);
        self.set_flag(Flag::H, (hl & 0x0FFF) + (b & 0x0FFF) > 0x0FFF);

        let msb = (r >> 8) as u8;
        let lsb = (r & 0xFF) as u8;

        self.h = msb;
        self.l = lsb;
    }

    pub fn alu_addsp(&mut self, b: u8) {
        // the magic of 2's complement: read as unsigned, extend to 16 bits and interpret as
        // unsigned. Then we do wrapping_add and it's the same as subtracting :mindblown:
        let val = b as i8 as i16 as u16;
        let r = (self.sp).wrapping_add(val);

        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::C, (self.sp & 0xFF) + (val & 0xFF) > 0xFF);
        self.set_flag(Flag::H, (self.sp & 0xF) + (val & 0xF) > 0xF);

        self.sp = r;
    }

    pub fn alu_adc(&mut self, b: u8) {
        self.alu_add_internal(b, true)
    }

    fn alu_sub_internal(&mut self, b: u8, with_carry: bool) {
        let a = self.a;
        let c = if self.get_flag(Flag::C) { 1 } else { 0 };
        let mut r = a.wrapping_sub(b);

        if with_carry {
            r = r.wrapping_sub(c);
        }

        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, true);

        if with_carry {
            self.set_flag(Flag::C, (a as i16) - (b as i16) - (c as i16) < 0);
            self.set_flag(Flag::H, ((a & 0xF) as i16) - ((b & 0xF) as i16) - (c as i16) < 0);
        } else {
            self.set_flag(Flag::C, (a as i16) - (b as i16) < 0);
            self.set_flag(Flag::H, ((a & 0xF) as i16) - ((b & 0xF) as i16) < 0);
        }

        self.a = r;
    }

    pub fn alu_sub(&mut self, b: u8) {
        self.alu_sub_internal(b, false)
    }

    pub fn alu_sbc(&mut self, b: u8) {
        self.alu_sub_internal(b, true)
    }

    pub fn alu_and(&mut self, b: u8) {
        let r = self.a & b;

        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, true);
        self.set_flag(Flag::C, false);

        self.a = r;
    }

    pub fn alu_xor(&mut self, b: u8) {
        let r = self.a ^ b;

        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, false);

        self.a = r;
    }

    pub fn alu_or(&mut self, b: u8) {
        let r = self.a | b;

        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, false);

        self.a = r;
    }

    pub fn alu_cp(&mut self, b: u8) {
        self.set_flag(Flag::Z, self.a == b);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::C, self.a < b);
        self.set_flag(Flag::H, (self.a & 0xF) < (b & 0xF));
    }

    pub fn alu_inc(&mut self, a: u8) -> u8 {
        let r = a.wrapping_add(1);
        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, (a & 0xF) == 0xF);

        r
    }

    pub fn alu_inc16(&mut self, a: u16) -> u16 {
        a.wrapping_add(1)
    }

    pub fn alu_dec(&mut self, a: u8) -> u8 {
        let r = a.wrapping_sub(1);
        self.set_flag(Flag::Z, r == 0);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, (a & 0xF) == 0);

        r
    }

    pub fn alu_dec16(&mut self, a: u16) -> u16 {
        a.wrapping_sub(1)
    }

    pub fn alu_cpl(&mut self) {
        self.a = !self.a;
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, true);
    }

    pub fn alu_ccf(&mut self) {
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, !self.get_flag(Flag::C));
    }

    pub fn alu_scf(&mut self) {
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, true);
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
