use minifb::Window;
use std::time;
use std::thread;

mod registers;
mod memory;
mod cpu;
mod gpu;

use cpu::CPU;

const NUMBER_OF_PIXELS: usize = 160*144;
// TODO check
const ONE_FRAME_IN_CYCLES: usize = 17556;

pub struct Emulator {
    cpu: CPU,
    window: minifb::Window,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: CPU::new(),
            window: minifb::Window::new(
                "gbemu-rs",
                160,
                144,
                minifb::WindowOptions::default())
                .expect("failed to create window"),
        }
    }

    pub fn run(&mut self) {
        let mut window_buffer: [u32; NUMBER_OF_PIXELS] = [0; NUMBER_OF_PIXELS];
        let mut cycles_elapsed = 0usize;
        let mut now = time::Instant::now();

        while self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape) {
            let delta = now.elapsed().subsec_nanos();
            now = time::Instant::now();

            cycles_elapsed += self.cpu.run(delta);
            if cycles_elapsed >= ONE_FRAME_IN_CYCLES {
                for (i, pixel) in self.cpu.pixel_buffer().enumerate() {
                    window_buffer[i] = (*pixel) as u32;
                }

                self.window.update_with_buffer(&window_buffer[..], 160, 144).unwrap();
                cycles_elapsed = 0;
            } else {
                thread::sleep(time::Duration::from_nanos(2));
            }
        }
    }
}
