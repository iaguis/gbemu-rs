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
    Nop,
    Ld16Rr,
    Ld16AI,
    Inc16,
    Inc8,
    Dec8,
    Ld8I,
    Jp = 0xc3,
}

impl TryFrom<u8> for Opcode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Opcode::Nop),
            0x01 => Ok(Opcode::Ld16Rr),
            0x02 => Ok(Opcode::Ld16AI),
            0x03 => Ok(Opcode::Inc16),
            0x04 => Ok(Opcode::Inc8),
            0x05 => Ok(Opcode::Dec8),
            0x06 => Ok(Opcode::Ld8I),
            0xc3 => Ok(Opcode::Jp),
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
        self.reg.pc += 1;
        println!("mem[pc] = {:#04x}", b);

        let opcode = Opcode::try_from(b)?;
        Ok(opcode)
    }

    fn execute(&mut self) -> u8 {
        // XXX this panics if it fails to decode the opcode, which is probably fine
        let opcode = self.fetch_byte().expect("failed fetching");

        println!("opcode: {:?}", opcode);
        let cycles = match opcode {
            Opcode::Nop => {
                println!("nop, sleeping 1s");
                1
            },
            Opcode::Ld16Rr => {
                println!("Executing Ld16Rr");
                self.reg.b = self.memory_bus.read_byte(self.reg.pc.into());
                self.reg.pc += 1;
                self.reg.c = self.memory_bus.read_byte(self.reg.pc.into());
                self.reg.pc += 1;

                println!("BC = {}", self.reg.bc());
                println!("B = {}", self.reg.b);
                println!("C = {}", self.reg.c);

                3
            },
            Opcode::Ld16AI => {
                println!("Executing Ld16AI");

                // TODO check
                self.reg.b = 0;
                self.reg.c = self.reg.a;

                println!("BC = {}", self.reg.bc());
                println!("B = {}", self.reg.b);
                println!("C = {}", self.reg.c);

                1
            },
            Opcode::Inc16 => {
                self.reg.inc_bc();
                println!("Executing Inc16");

                1
            },
            Opcode::Inc8 => {
                self.reg.inc_b();
                println!("Executing Inc8");

                1
            },
            Opcode::Dec8 => {
                self.reg.dec_b();
                println!("Executing Dec8");

                1
            },
            Opcode::Jp => {
                let address_lo = self.memory_bus.read_byte(self.reg.pc.into()) as u16;
                self.reg.pc += 1;
                let address_hi = (self.memory_bus.read_byte(self.reg.pc.into()) as u16) << 8;
                self.reg.pc += 1;
                let address = address_hi | address_lo;

                println!("Executing Jp to {:#04x}", address);

                self.reg.pc = address;

                4
            },
            Opcode::Ld8I => {
                self.reg.b = self.memory_bus.read_byte(self.reg.pc.into());
                self.reg.pc += 1;

                2
            }
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
