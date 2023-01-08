use std::env;

use std::time::{Instant,Duration};
use std::thread::sleep;

mod registers;
mod memory;
mod memory_bus;
mod cpu;
mod gpu;
mod io;
mod debug;

use cpu::CPU;

const NUMBER_OF_PIXELS: usize = 160*144 + 1;
// TODO check

pub struct Emulator {
    cpu: CPU,
    window: minifb::Window,
}


const ONE_SECOND_IN_MICROS: usize = 1000000000;
const ONE_SECOND_IN_CYCLES: usize = 4190000;
const ONE_FRAME_IN_CYCLES: usize = 70224;

impl Emulator {
    pub fn new(config: Config) -> Emulator {
        let mut window_options = minifb::WindowOptions::default();
        window_options.scale = minifb::Scale::X4;
        Emulator {
            cpu: CPU::new(config.rom_path, config.debug),
            window: minifb::Window::new(
                "gbemu-rs",
                160,
                144,
                window_options)
                .expect("failed to create window"),
        }
    }

    pub fn run(&mut self) {
        let mut window_buffer: [u32; NUMBER_OF_PIXELS+1] = [0; NUMBER_OF_PIXELS+1];
        let mut cycles_elapsed_in_frame = 0usize;
        let mut now = Instant::now();

        while self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape) {
            if self.window.is_key_down(minifb::Key::Space) {
                self.cpu.stop_at_next_frame = true;
            }

            let time_delta = now.elapsed().subsec_nanos();
            now = Instant::now();
            let delta = time_delta as f64 / ONE_SECOND_IN_MICROS as f64;
            let cycles_to_run = delta * ONE_SECOND_IN_CYCLES as f64;

            let mut cycles_elapsed = 0;

            while cycles_elapsed <= cycles_to_run as usize {
                cycles_elapsed += self.cpu.step() as usize;
            }
            cycles_elapsed_in_frame += cycles_elapsed;

            if cycles_elapsed_in_frame >= ONE_FRAME_IN_CYCLES {
                if self.cpu.stop_at_next_frame {
                    self.cpu.drop_to_shell();
                }

                for (i, pixel) in self.cpu.pixel_buffer().enumerate() {
                    window_buffer[i] = *pixel;
                }
                self.window.update_with_buffer(&window_buffer[..], 160, 144).unwrap();
                cycles_elapsed_in_frame = 0;
            } else {
                sleep(Duration::from_nanos(2))
            }
        }
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
