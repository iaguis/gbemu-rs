use std::env;

use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Instant,Duration};
use std::thread::sleep;
use minifb::{Window,Key,WindowOptions,Scale};

mod registers;
mod memory;
mod memory_bus;
mod cpu;
mod gpu;
mod keys;
mod debug;

use cpu::CPU;

const NUMBER_OF_PIXELS: usize = 160*144 + 1;

pub struct Emulator {
    cpu: CPU,
    window: Window,
}


const ONE_SECOND_IN_NANOS: usize = 1000000000;
const ONE_SECOND_IN_CYCLES: usize = 4190000;
const ONE_FRAME_IN_CYCLES: usize = 70224;

pub struct KeyData {
    key: minifb::Key,
    state: bool,
}

// TODO understand this thing :D
// adapted from https://github.com/emoon/rust_minifb/blob/master/examples/char_callback.rs
type KeyVec = Rc<RefCell<Vec<KeyData>>>;

pub struct KeysCallback {
    keys: KeyVec
}

impl KeysCallback {
    fn new(data: &KeyVec) -> KeysCallback {
        KeysCallback{
            keys: data.clone(),
        }
    }
}

impl minifb::InputCallback for KeysCallback {
    fn add_char(&mut self, _uni_char: u32) {}

    fn set_key_state(&mut self, _key: minifb::Key, _state: bool) {
        self.keys.borrow_mut().push(KeyData{ key: _key, state: _state })
    }
}

impl Emulator {
    pub fn new(config: Config) -> Emulator {
        let mut window_options = WindowOptions::default();
        window_options.scale = Scale::X8;

        Emulator {
            cpu: CPU::new(config.rom_path, config.boot_rom_path, config.debug),
            window: Window::new(
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

        self.window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        let keys_data = KeyVec::new(RefCell::new(Vec::new()));

        let keys_callback = Box::new(KeysCallback::new(&keys_data));

        self.window.set_input_callback(keys_callback);

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            if self.window.is_key_down(Key::Space) {
                self.cpu.stop_at_next_frame = true;
            }

            let time_delta = now.elapsed().subsec_nanos();
            now = Instant::now();
            let delta = time_delta as f64 / ONE_SECOND_IN_NANOS as f64;
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

                let mut keys = keys_data.borrow_mut();

                for k in keys.iter() {
                    if k.state {
                        self.cpu.memory_bus.joypad.key_down(&k.key);
                    } else {
                        self.cpu.memory_bus.joypad.key_up(&k.key);
                    }
                }

                keys.clear();

                cycles_elapsed_in_frame = 0;
            } else {
                sleep(Duration::from_nanos(2))
            }
        }
    }
}

pub struct Config {
    pub rom_path: String,
    pub boot_rom_path: Option<String>,
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

        let boot_rom_path = args.next();

        let debug = env::var("GBEMU_RS_DEBUG").is_ok();

        Ok(Config {
            rom_path,
            boot_rom_path,
            debug,
        })
    }
}
