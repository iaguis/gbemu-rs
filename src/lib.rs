use std::env;

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

impl Emulator {
    pub fn new(config: Config) -> Emulator {
        Emulator {
            cpu: CPU::new(config.rom_path, config.debug),
            window: minifb::Window::new(
                "gbemu-rs",
                160,
                144,
                minifb::WindowOptions::default())
                .expect("failed to create window"),
        }
    }

    pub fn run(&mut self) {
        let mut window_buffer: [u32; NUMBER_OF_PIXELS+1] = [0; NUMBER_OF_PIXELS+1];

        while self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape) {
            self.cpu.frame();

            for (i, pixel) in self.cpu.pixel_buffer().enumerate() {
                window_buffer[i] = *pixel;
            }

            self.window.update_with_buffer(&window_buffer[..], 160, 144).unwrap();
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
