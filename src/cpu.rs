use std::thread;
use std::time;
use std::fs::File;

use crate::registers::{Flag,Registers};
use crate::memory_bus::MemoryBus;

pub struct CPU {
    pub reg: Registers,
    pub memory_bus: MemoryBus,
    pub counter: i32,
    pub tmp_buffer: Vec<u8>,
    IME: bool,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Opcode {
    NOP,
    LD(LDType),
    INC(IncDecTarget),
    DEC(IncDecTarget),
    PUSH(StackTarget),
    POP(StackTarget),
    JP(JCondition),
    JR(JCondition),
    DI,
}

#[derive(Debug)]
pub enum JCondition {
    Nothing,
    NZ,
    NC,
    Z,
    C,
}

#[derive(Debug)]
pub enum LDType {
    Byte(LDTarget, LDSource),
    Word(LDWordTarget),
    AFromIndirect(Indirect),
    IndirectFromA(Indirect),
    AFromAddress,
    AddressFromA,
    SPFromHL,
    IndirectFromSP,
}

#[derive(Debug)]
pub enum IncDecTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    BC,
    DE,
    HL,
    SP,
    HLIndirect,
}

#[derive(Debug)]
pub enum Indirect {
    BCIndirect,
    DEIndirect,
    HLIndirectInc,
    HLIndirectDec,
    LastByteIndirect,
}

#[derive(Debug)]
pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
}

#[derive(Debug)]
pub enum LDWordTarget {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug)]
pub enum LDSource {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    D8,
    HLIndirect,
}

#[derive(Debug)]
pub enum LDTarget {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    HLIndirect,
}

impl TryFrom<u8> for Opcode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Opcode::NOP),
            0x01 => Ok(Opcode::LD(LDType::Word(LDWordTarget::BC))),
            0x02 => Ok(Opcode::LD(LDType::IndirectFromA(Indirect::BCIndirect))),
            0x03 => Ok(Opcode::INC(IncDecTarget::BC)),
            0x04 => Ok(Opcode::INC(IncDecTarget::B)),
            0x05 => Ok(Opcode::DEC(IncDecTarget::B)),
            0x06 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::D8))),

            0x08 => Ok(Opcode::LD(LDType::IndirectFromSP)),

            0x0a => Ok(Opcode::LD(LDType::AFromIndirect(Indirect::BCIndirect))),
            0x0b => Ok(Opcode::DEC(IncDecTarget::BC)),
            0x0c => Ok(Opcode::INC(IncDecTarget::C)),
            0x0d => Ok(Opcode::DEC(IncDecTarget::C)),
            0x0e => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::D8))),


            0x11 => Ok(Opcode::LD(LDType::Word(LDWordTarget::DE))),
            0x12 => Ok(Opcode::LD(LDType::IndirectFromA(Indirect::DEIndirect))),
            0x13 => Ok(Opcode::INC(IncDecTarget::DE)),
            0x14 => Ok(Opcode::INC(IncDecTarget::D)),
            0x15 => Ok(Opcode::DEC(IncDecTarget::D)),
            0x16 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::D8))),

            0x18 => Ok(Opcode::JR(JCondition::Nothing)),

            0x1a => Ok(Opcode::LD(LDType::AFromIndirect(Indirect::DEIndirect))),
            0x1b => Ok(Opcode::DEC(IncDecTarget::DE)),
            0x1c => Ok(Opcode::INC(IncDecTarget::E)),
            0x1d => Ok(Opcode::DEC(IncDecTarget::E)),
            0x1e => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::D8))),

            0x20 => Ok(Opcode::JR(JCondition::NZ)),
            0x21 => Ok(Opcode::LD(LDType::Word(LDWordTarget::HL))),
            0x22 => Ok(Opcode::LD(LDType::IndirectFromA(Indirect::HLIndirectInc))),
            0x23 => Ok(Opcode::INC(IncDecTarget::HL)),
            0x24 => Ok(Opcode::INC(IncDecTarget::H)),
            0x25 => Ok(Opcode::DEC(IncDecTarget::H)),
            0x26 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::D8))),

            0x28 => Ok(Opcode::JR(JCondition::Z)),


            0x2a => Ok(Opcode::LD(LDType::AFromIndirect(Indirect::HLIndirectInc))),
            0x2b => Ok(Opcode::DEC(IncDecTarget::HL)),
            0x2c => Ok(Opcode::INC(IncDecTarget::L)),
            0x2d => Ok(Opcode::DEC(IncDecTarget::L)),
            0x2e => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::D8))),

            0x31 => Ok(Opcode::LD(LDType::Word(LDWordTarget::SP))),
            0xc3 => Ok(Opcode::JP(JCondition::Nothing)),
            0xea => Ok(Opcode::LD(LDType::AddressFromA)),
            0xf3 => Ok(Opcode::DI),
            _ => Err("unknown opcode"),
        }
    }
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            reg: Registers::new(),
            counter: 20,
            memory_bus: MemoryBus::new(),
            // TODO remove
            tmp_buffer: vec![1; 100],
            IME: true,
        };

        // TODO error handling

        // FIXME pass this from main
        let f = File::open("/home/iaguis/programming/gameboy/cpu_instrs/cpu_instrs.gb").expect("can't open ROM");
        cpu.memory_bus.read_rom(f).expect("can't read ROM");

        cpu
    }

    fn fetch_byte(&mut self) -> Result<Opcode, &'static str> {
        println!("pc = {:#04x}", self.reg.pc);
        let b = self.memory_bus.read_byte(self.reg.pc.into());
        println!("mem[pc] = {:#04x}", b);

        let opcode = Opcode::try_from(b)?;
        Ok(opcode)
    }

    // TODO double-check cycles
    fn execute(&mut self) -> u8 {
        // XXX this panics if it fails to decode the opcode, which is probably fine
        let opcode = self.fetch_byte().expect("failed fetching");

        let mut cycles = 1;

        println!("opcode: {:?}", opcode);
        match opcode {
            Opcode::NOP => {
                println!("nop, sleeping 1s");
                cycles = 1;
                self.reg.pc += 1;
            },

            Opcode::LD(ld_type) => {
                match ld_type {
                    LDType::Byte(target, source) => {
                        let source_val = match source {
                            LDSource::A => self.reg.a,
                            LDSource::B => self.reg.b,
                            LDSource::C => self.reg.c,
                            LDSource::D => self.reg.d,
                            LDSource::E => self.reg.e,
                            LDSource::F => self.reg.f,
                            LDSource::H => self.reg.h,
                            LDSource::L => self.reg.l,
                            LDSource::D8 => self.memory_bus.read_byte(self.reg.pc + 1),
                            LDSource::HLIndirect => self.memory_bus.read_byte(self.reg.hl()),
                        };
                        match target {
                            LDTarget::A => self.reg.a = source_val,
                            LDTarget::B => self.reg.b = source_val,
                            LDTarget::C => self.reg.c = source_val,
                            LDTarget::D => self.reg.d = source_val,
                            LDTarget::E => self.reg.e = source_val,
                            LDTarget::F => self.reg.f = source_val,
                            LDTarget::H => self.reg.h = source_val,
                            LDTarget::L => self.reg.l = source_val,
                            LDTarget::HLIndirect => {
                                self.memory_bus.write_byte(self.reg.hl(), source_val)
                            }
                        }

                        match source {
                            LDSource::D8 => {cycles = 2; self.reg.pc += 2},
                            LDSource::HLIndirect => {cycles = 1; self.reg.pc += 2 },
                            _ => {cycles = 1; self.reg.pc += 1}
                        }
                    },
                    LDType::Word(ld_word_target) => {
                        // little-endian
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        match ld_word_target {
                            LDWordTarget::BC => {
                                self.reg.b = msb;
                                self.reg.c = lsb;
                            },
                            LDWordTarget::DE => {
                                self.reg.d = msb;
                                self.reg.e = lsb;
                            },
                            LDWordTarget::HL => {
                                self.reg.h = msb;
                                self.reg.l = lsb;
                            },
                            LDWordTarget::SP => {
                                self.reg.sp = ((msb as u16) << 8) | lsb as u16
                            },
                        }

                        cycles = 3;
                        self.reg.pc += 3;
                    },
                    LDType::IndirectFromA(indirect) => {
                        let a = self.reg.a;

                        match indirect {
                            Indirect::BCIndirect => {
                                let bc = self.reg.bc();
                                self.memory_bus.write_byte(bc, a);
                            }
                            Indirect::DEIndirect => {
                                let de = self.reg.de();
                                self.memory_bus.write_byte(de, a);
                            }
                            Indirect::HLIndirectInc => {
                                let r = self.reg.alu_inc16(self.reg.hl());
                                self.reg.set_hl(r);

                                self.memory_bus.write_byte(r, a);
                            }
                            Indirect::HLIndirectDec => {
                                let r = self.reg.alu_dec16(self.reg.hl());
                                self.reg.set_hl(r);

                                self.memory_bus.write_byte(r, a);
                            }
                            Indirect::LastByteIndirect => {
                                let c = self.reg.c as u16;
                                self.memory_bus.write_byte(0xFF00 + c, a);
                            }
                        }

                        cycles = 2;
                        self.reg.pc += 1;
                    },
                    LDType::AddressFromA => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        let address = ((msb as u16) << 8) | lsb as u16;

                        println!("address {:#04x}", address);

                        self.memory_bus.write_byte(address, self.reg.a);

                        cycles = 4;
                        self.reg.pc += 3;
                    },

                    LDType::AFromAddress => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        let address = ((msb as u16) << 8) | lsb as u16;

                        println!("address {:#04x}", address);

                        self.reg.a = self.memory_bus.read_byte(address);

                        cycles = 4;
                        self.reg.pc += 3;
                    },

                    LDType::AFromIndirect(indirect) => {
                        match indirect {
                            Indirect::BCIndirect => {
                                let bc = self.reg.bc();
                                self.reg.a = self.memory_bus.read_byte(bc);
                            }
                            Indirect::DEIndirect => {
                                let de = self.reg.de();
                                self.reg.a = self.memory_bus.read_byte(de);
                            }
                            Indirect::HLIndirectInc => {
                                let r = self.reg.alu_inc16(self.reg.hl());
                                self.reg.set_hl(r);

                                self.reg.a = self.memory_bus.read_byte(r);
                            }
                            Indirect::HLIndirectDec => {
                                let r = self.reg.alu_dec16(self.reg.hl());
                                self.reg.set_hl(r);

                                self.reg.a = self.memory_bus.read_byte(r);
                            }
                            Indirect::LastByteIndirect => {
                                let c = self.reg.c as u16;
                                self.reg.a = self.memory_bus.read_byte(0xFF00 + c);
                            }
                        }

                        cycles = 2;
                        self.reg.pc += 1;
                    },

                    LDType::SPFromHL => {
                        self.reg.sp = self.reg.hl();

                        cycles = 2;
                        self.reg.pc += 1;
                    },

                    LDType::IndirectFromSP => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        let address = ((msb as u16) << 8) | lsb as u16;

                        self.memory_bus.write_byte(address, (self.reg.sp & 0xff) as u8);
                        self.memory_bus.write_byte(address+1, (self.reg.sp >> 8) as u8);

                        cycles = 5;
                        self.reg.pc += 3;
                    },
                }
            },

            Opcode::INC(target) => {
                match target {
                    IncDecTarget::A => { self.reg.a = self.reg.alu_inc(self.reg.a); },
                    IncDecTarget::B => { self.reg.b = self.reg.alu_inc(self.reg.b); },
                    IncDecTarget::C => { self.reg.c = self.reg.alu_inc(self.reg.c); },
                    IncDecTarget::D => { self.reg.d = self.reg.alu_inc(self.reg.d); },
                    IncDecTarget::E => { self.reg.e = self.reg.alu_inc(self.reg.e); },
                    IncDecTarget::H => { self.reg.h = self.reg.alu_inc(self.reg.h); },
                    IncDecTarget::L => { self.reg.l = self.reg.alu_inc(self.reg.l); },
                    IncDecTarget::BC => {
                        let r = self.reg.alu_inc16(self.reg.bc());
                        self.reg.set_bc(r);
                    },
                    IncDecTarget::DE => {
                        let r = self.reg.alu_inc16(self.reg.de());
                        self.reg.set_de(r);
                    },
                    IncDecTarget::HL => {
                        let r = self.reg.alu_inc16(self.reg.hl());
                        self.reg.set_hl(r);
                    },
                    IncDecTarget::SP => {
                        let r = self.reg.alu_inc16(self.reg.sp);
                        self.reg.sp = r;
                    },
                    IncDecTarget::HLIndirect => {
                        let val = self.memory_bus.read_byte(self.reg.hl());
                        let r = self.reg.alu_dec(val);
                        self.memory_bus.write_byte(self.reg.hl(), r);
                    },
                }

                match target {
                    IncDecTarget::HLIndirect => { cycles = 3; },
                    IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => { cycles = 2; },
                    _ => { cycles = 1; },
                }
                self.reg.pc += 1;
            },

            Opcode::DEC(target) => {
                match target {
                    IncDecTarget::A => { self.reg.a = self.reg.alu_dec(self.reg.a); },
                    IncDecTarget::B => { self.reg.b = self.reg.alu_dec(self.reg.b); },
                    IncDecTarget::C => { self.reg.c = self.reg.alu_dec(self.reg.c); },
                    IncDecTarget::D => { self.reg.d = self.reg.alu_dec(self.reg.d); },
                    IncDecTarget::E => { self.reg.e = self.reg.alu_dec(self.reg.e); },
                    IncDecTarget::H => { self.reg.h = self.reg.alu_dec(self.reg.h); },
                    IncDecTarget::L => { self.reg.l = self.reg.alu_dec(self.reg.l); },
                    IncDecTarget::BC => {
                        let r = self.reg.alu_dec16(self.reg.bc());
                        self.reg.set_bc(r);
                    },
                    IncDecTarget::DE => {
                        let r = self.reg.alu_dec16(self.reg.de());
                        self.reg.set_de(r);
                    },
                    IncDecTarget::HL => {
                        let r = self.reg.alu_dec16(self.reg.hl());
                        self.reg.set_hl(r);
                    },
                    IncDecTarget::SP => {
                        let r = self.reg.alu_dec16(self.reg.sp);
                        self.reg.sp = r;
                    },
                    IncDecTarget::HLIndirect => {
                        let val = self.memory_bus.read_byte(self.reg.hl());
                        let r = self.reg.alu_dec(val);
                        self.memory_bus.write_byte(self.reg.hl(), r);
                    },
                }

                match target {
                    IncDecTarget::HLIndirect => { cycles = 3; },
                    IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => { cycles = 2; },
                    _ => { cycles = 1 },
                }
                self.reg.pc += 1;
            },

            Opcode::JP(condition) => {
                let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                let jp_address = ((msb as u16) << 8) | (lsb as u16);

                match condition {
                    JCondition::Nothing => {
                        self.reg.pc = jp_address;
                        cycles = 4;
                    },
                    JCondition::NZ => {
                        if !self.reg.get_flag(Flag::Z) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::NC => {
                        if !self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::Z => {
                        if self.reg.get_flag(Flag::Z) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::C => {
                        if self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                }
            },

            Opcode::JR(condition) => {
                let offset = self.memory_bus.read_byte(self.reg.pc + 1) as i8;
                self.reg.pc += 1;

                let jp_address = (self.reg.pc + 1).wrapping_add(offset as u16);

                match condition {
                    JCondition::Nothing => {
                        self.reg.pc = jp_address;
                        cycles = 4;
                    },
                    JCondition::NZ => {
                        if !self.reg.get_flag(Flag::Z) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::NC => {
                        if !self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::Z => {
                        if self.reg.get_flag(Flag::Z) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::C => {
                        if self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                }
            },

            Opcode::DI => {
                self.IME = false;
                self.reg.pc += 1;
            },
            _ => { panic!("not implemented"); },
        };

        println!("{} cycles", cycles);
        cycles
    }

    // TODO implement
    pub fn pixel_buffer(&self) -> std::slice::Iter<'_, u8> {
        self.tmp_buffer.iter()
    }

    fn calculate_cycles(duration: u32) -> i32 {
        // XXX this might panic
        (duration/1000).try_into().unwrap()
    }

    pub fn run(&mut self, duration: u32) -> usize {
        let mut cycles_to_run = CPU::calculate_cycles(duration);
        let mut cycles_ran = 0;

        while self.counter > 0 {
            println!("emulating...");

            let cycles = self.execute();

            self.counter -= cycles as i32;
            cycles_to_run -= cycles as i32;
            println!("self.counter = {}", cycles);

            cycles_ran += cycles as usize;

            if cycles_to_run <= 0 {
                break;
            }

            if self.counter <= 0 {
                // TODO run tasks
                self.counter = 20;
                println!("running interrupts");
            }
        }

        cycles_ran
    }
}
