use gbemu_rs::{Emulator,Config};

use winit::event_loop::{ControlFlow, EventLoop};
use winit::dpi::LogicalSize;
use winit::window::WindowBuilder;
use pixels::{PixelsBuilder, SurfaceTexture};

use std::time::Instant;
use std::{env,process};

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {err}");
        process::exit(1);
    });

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(1280 as f64, 1152 as f64);
        WindowBuilder::new()
        .with_title("gbemu-rs")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(160, 144, surface_texture)
                          .enable_vsync(true)
                          .build().expect("failed to create pixels")
    };

    let mut e = Emulator::new(config);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            winit::event::Event::MainEventsCleared => {
                let mut frame = pixels.get_frame_mut();

                let now = Instant::now();
                let buffer = e.frame(now);

                let mut buffer_i = 0;
                for (_, pixel) in (&mut frame).chunks_exact_mut(4).enumerate() {
                    let argb = buffer[buffer_i];
                    buffer_i += 1;

                    let rgba = [((argb & 0x00FF0000) >> 16) as u8, ((argb & 0x0000FF00) >> 8) as u8, (argb & 0x000000FF) as u8, ((argb & 0xFF000000) >> 24) as u8];
                    pixel.copy_from_slice(&rgba);
                }
                if let Err(err) = pixels.render() {
                    println!("pixels.render() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            winit::event::Event::WindowEvent {
                event, ..
            } => match event {
                winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                winit::event::WindowEvent::KeyboardInput {
                    device_id: _, input: kin, is_synthetic: _
                } => {
                    match kin.virtual_keycode {
                        Some(k) => {
                            match kin.state {
                                winit::event::ElementState::Pressed => e.cpu.memory_bus.joypad.key_down(&k),
                                winit::event::ElementState::Released => e.cpu.memory_bus.joypad.key_up(&k),
                            }
                        },
                        _ => {},
                    }
                },
                _ => {},
            },
            _ => {}
        }
    });
}
