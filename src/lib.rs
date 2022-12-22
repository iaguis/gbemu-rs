use std::thread;
use std::time;

pub struct Emulator {
    cpu: Cpu,
    counter: u8,
}

struct Cpu {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,

    sp: u16,
    pc: u16,

    m: u8,
    t: u8,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu {
                af: 0,
                bc: 0,
                de: 0,
                hl: 0,
                sp: 0,
                pc: 0,
                m: 0,
                t: 0,
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

            let opcode = self.read_byte(self.cpu.pc);
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
