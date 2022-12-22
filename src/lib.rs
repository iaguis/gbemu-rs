use std::thread;
use std::time;

mod registers;

use registers::Registers;

pub struct Emulator {
    cpu: Cpu,
    counter: u8,
}

struct Cpu {
    reg: Registers
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu {
                reg: Registers::new()
            },
            counter: 0,
        }
    }

    fn read_word(&self, address: u16) -> u8 {
        0
    }

    fn write_word(&mut self, address: u16, val: u8) -> Result<(), &'static str> {
        Ok(())
    }

    fn get_cycles(&self, opcode: u8) -> u8 {
        0
    }

    fn execute(&self, opcode: u8) {
    }

    pub fn start(&mut self) {
        loop {
            println!("emulating...");

            let opcode = self.read_byte(self.cpu.reg.pc);
            self.counter -= self.get_cycles(opcode);

            self.execute(opcode);

            if self.counter <= 0 {
                // TODO run tasks
                self.counter += 10; // XXX interrupt period
            }

            thread::sleep(time::Duration::from_secs(5));
        }
    }
}
