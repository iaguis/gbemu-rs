use std::env;

use std::time::Instant;

mod registers;
mod memory;
mod memory_bus;
mod cpu;
mod gpu;
mod keys;
mod debug;

use cpu::CPU;

pub struct Emulator {
    pub cpu: CPU,
}

const ONE_SECOND_IN_NANOS: usize = 1000000000;
const ONE_SECOND_IN_CYCLES: usize = 4190000;
const ONE_FRAME_IN_CYCLES: usize = 70224;

impl Emulator {
    pub fn new(config: Config) -> Emulator {
        Emulator {
            cpu: CPU::new(config.rom_path, config.debug),
        }
    }

    pub fn frame(&mut self, mut now: Instant) -> Vec<u32> {
        let mut cycles_elapsed_in_frame = 0usize;

        while cycles_elapsed_in_frame < ONE_FRAME_IN_CYCLES {
            let time_delta = now.elapsed().subsec_nanos();
            now = Instant::now();
            let delta = time_delta as f64 / ONE_SECOND_IN_NANOS as f64;
            let cycles_to_run = delta * ONE_SECOND_IN_CYCLES as f64;

            let mut cycles_elapsed = 0;

            while cycles_elapsed <= cycles_to_run as usize {
                cycles_elapsed += self.cpu.step() as usize;
            }
            cycles_elapsed_in_frame += cycles_elapsed;
        }

        if self.cpu.stop_at_next_frame {
            self.cpu.drop_to_shell();
        }

        self.cpu.pixel_buffer()
    }
}

pub struct Config {
    pub rom_path: String,
    pub debug: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // program name
        args.next();

        let rom_path = match args.next() {
            Some(arg) => arg,
            None => return Err("missing rom path"),
        };

        let debug = env::var("GBEMU_RS_DEBUG").is_ok();

        Ok(Config {
            rom_path,
            debug,
        })
    }
}
