use std::thread;
use std::time;
use std::fs::File;

use crate::registers::Registers;
use crate::memory_bus::MemoryBus;

pub struct CPU {
    pub reg: Registers,
    pub memory_bus: MemoryBus,
    pub counter: i32,
    pub tmp_buffer: Vec<u8>,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Opcode {
    NOP,
    LD(LDType),
    INC(IncTarget),
    DEC(IncTarget),
    PUSH(StackTarget),
    POP(StackTarget),
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
pub enum IncTarget {
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
            0x03 => Ok(Opcode::INC(IncTarget::BC)),
            0x04 => Ok(Opcode::INC(IncTarget::B)),
            0x05 => Ok(Opcode::DEC(IncTarget::B)),
            0x06 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::D8))),
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
                                let hl = self.reg.hl();
                                self.reg.set_hl(hl.wrapping_add(1));
                                self.memory_bus.write_byte(hl, a);
                            }
                            Indirect::HLIndirectDec => {
                                let hl = self.reg.hl();
                                self.reg.set_hl(hl.wrapping_sub(1));
                                self.memory_bus.write_byte(hl.wrapping_sub(1), a);
                            }
                            Indirect::LastByteIndirect => {
                                let c = self.reg.c as u16;
                                self.memory_bus.write_byte(0xFF00 + c, a);
                            }
                        }

                        cycles = 2;
                        self.reg.pc += 1;
                    }
                    _ => { panic!("not implemented"); }
                }
            },
            Opcode::INC(target) => {
                match target {
                    IncTarget::A => { self.reg.a.wrapping_add(1); },
                    IncTarget::B => { self.reg.b.wrapping_add(1); },
                    IncTarget::C => { self.reg.c.wrapping_add(1); },
                    IncTarget::D => { self.reg.d.wrapping_add(1); },
                    IncTarget::E => { self.reg.e.wrapping_add(1); },
                    IncTarget::H => { self.reg.h.wrapping_add(1); },
                    IncTarget::L => { self.reg.l.wrapping_add(1); },
                    IncTarget::BC => { self.reg.inc_bc(); },
                    IncTarget::DE => {
                        panic!("not implemented");
                    },
                    IncTarget::HL => {
                        panic!("not implemented");
                    },
                    IncTarget::SP => {
                        panic!("not implemented");
                    },
                    IncTarget::HLIndirect => {
                        panic!("not implemented");
                    },
                }
                cycles = 1;
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
