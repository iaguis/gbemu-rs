use std::thread;
use std::time;
use std::fs::File;

mod registers;
mod memory;

use registers::Registers;
use memory::Memory;

pub struct Emulator {
    cpu: Cpu,
}

struct Cpu {
    reg: Registers,
    counter: i32,
    memory: Memory,
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
            0xc3 => Ok(Opcode::Jp),
            _ => Err("unknown opcode"),
        }
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Cpu {
            reg: Registers::new(),
            counter: 20,
            memory: Memory::new(),
        };

        // TODO error handling

        // FIXME pass this from main
        let mut f = File::open("/home/iaguis/programming/gameboy/cpu_instrs/cpu_instrs.gb").expect("can't open ROM");
        cpu.memory.read_rom(f).expect("can't read ROM");

        cpu
    }

    fn read_byte(&self, address: usize) -> u8 {
        assert!(address < self.memory.rom_0.len() * 2);

        match address {
            0..=0x3FFE => self.memory.rom_0[address],
            0x3FFF..=0x7FFE => self.memory.rom_n[address-0x3FFF],
            // FIXME
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: u16, val: u8) -> Result<(), &'static str> {
        Ok(())
    }

    fn fetch_byte(&mut self) -> Result<Opcode, &'static str> {
        println!("pc = {:#04x}", self.reg.pc);
        let b = self.read_byte(self.reg.pc.into());
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
                self.reg.b = self.read_byte(self.reg.pc.into());
                self.reg.pc += 1;
                self.reg.c = self.read_byte(self.reg.pc.into());
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
                let address_lo = self.read_byte(self.reg.pc.into()) as u16;
                self.reg.pc += 1;
                let address_hi = (self.read_byte(self.reg.pc.into()) as u16) << 8;
                self.reg.pc += 1;
                let address = address_hi | address_lo;

                println!("Executing Jp to {:#04x}", address);

                self.reg.pc = address;

                4
            }
        };

        println!("{} cycles", cycles);
        cycles
    }

    fn run(&mut self) {
        loop {
            println!("emulating...");

            let cycles = self.execute();

            self.counter -= cycles as i32;
            println!("self.counter = {}", cycles);

            if self.counter <= 0 {
                // TODO run tasks
                println!("running interrupts");
                self.counter += 20; // XXX interrupt period
            }
        }
    }
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu::new()
        }
    }

    fn get_cycles(&self, opcode: u8) -> u8 {
        0
    }

    pub fn start(&mut self) {
        self.cpu.run()
    }
}
