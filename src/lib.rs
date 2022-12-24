use std::thread;
use std::time;

mod registers;

use registers::Registers;

pub struct Emulator {
    cpu: Cpu,
}

struct Cpu {
    reg: Registers,
    counter: u8,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Opcode {
    Nop,
    Ld16Rr,
}

impl TryFrom<u8> for Opcode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Opcode::Nop),
            0x01 => Ok(Opcode::Ld16Rr),
            _ => Err("unknown opcode"),
        }
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            reg: Registers::new(),
            counter: 0,
        }
    }

    fn read_byte(&self, address: usize) -> u8 {
        let code: [u8; 16] = [0x01, 0xaa, 0xbb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        assert!(address < code.len());

        code[address]
    }

    fn write_byte(&mut self, address: u16, val: u8) -> Result<(), &'static str> {
        Ok(())
    }

    fn fetch_byte(&mut self) -> Result<Opcode, &'static str> {
        let b = self.read_byte(self.reg.pc.into());
        self.reg.pc += 1;
        println!("mem[pc] = {}", b);

        let opcode = Opcode::try_from(b)?;
        Ok(opcode)
    }

    fn execute(&mut self) {
        // XXX this panics if it fails to decode the opcode, which is probably fine
        let opcode = self.fetch_byte().expect("failed fetching");

        println!("opcode: {:?}", opcode);
        match opcode {
            Opcode::Nop => {
                println!("nop, sleeping 1s");
                thread::sleep(time::Duration::from_secs(1));
            },
            Opcode::Ld16Rr => {
                println!("Executing Ld16Rr");
                self.reg.b = self.read_byte(self.reg.pc.into()) as u8;
                self.reg.pc += 1;
                self.reg.c = self.read_byte(self.reg.pc.into()) as u8;
                self.reg.pc += 1;

                println!("BC = {}", self.reg.bc());
                println!("B = {}", self.reg.b);
                println!("C = {}", self.reg.c);

                thread::sleep(time::Duration::from_secs(1));
            },
        };
    }

    fn run(&mut self) {
        loop {
            println!("emulating...");

            self.execute();

            if self.counter <= 0 {
                // TODO run tasks
                self.counter += 10; // XXX interrupt period
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
