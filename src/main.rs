mod chip8;
mod opcode_parser;
mod fstools;
mod input;
mod audio;
mod args;

use std::sync::{Arc, RwLock};

use crate::args::Rgb;
use crate::fstools::get_file_as_byte_vec;
use crate::chip8::Chip8;

#[macro_use]
extern crate savefile_derive;
extern crate savefile;

use input::parse_input;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {

    // args
    let flags = crate::args::parse_args();

    // setup speed
    // devide 2 as fetch and decode is on same loop
    let runhz:u64 = flags.hz;
    let delay:u64 = 1000/runhz;
    let satisfiedruntimes: u64 = (1000/60)/delay;

    // setup cpu instance
    let mut chip8inst = Chip8::new();
    chip8inst.display = [flags.invert_colors; 2048];

    // load rom/state into chip8inst
    let rompath = flags.rom_path.as_str();
    if rompath.ends_with(".state") {
        crate::fstools::load_state(&std::path::Path::new(rompath).to_path_buf(), &mut chip8inst)
    }
    else {
        chip8inst.load_program(&get_file_as_byte_vec(rompath));
    }
    let chip8arc = Arc::new(RwLock::new(chip8inst));

    let loopchip8 = chip8arc.clone();
    std::thread::spawn(move || {
        let beeper = crate::audio::Beeper::new(flags.vol);
        let beeperexist = beeper.is_ok() && flags.vol > 0.0;
        if !beeperexist {
            println!("Audio not initialized!");
        }

        let mut runtimes = 0;
        loop {
            let next_frame_time = std::time::Instant::now() + std::time::Duration::from_millis(delay);
            
            // timer stuff
            if runtimes >= satisfiedruntimes {
                if loopchip8.read().unwrap().delay_timer > 0 {
                    loopchip8.write().unwrap().delay_timer -= 1;
                }
                if loopchip8.read().unwrap().sound_timer > 0 {
                    if beeperexist {
                        beeper.as_ref().unwrap().play();
                    }
                    loopchip8.write().unwrap().sound_timer -= 1;
                }
                else if beeperexist {
                    beeper.as_ref().unwrap().pause();
                }

                runtimes = 0;
            }
            runtimes += 1;


            // cycle cpu
            loopchip8.write().unwrap().single_cycle();

            if next_frame_time > std::time::Instant::now() {
                std::thread::sleep(next_frame_time - std::time::Instant::now());
            }
        }
    });

    // setup opengl
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(64, 32, surface_texture).unwrap()
    };

    let eventloopchip8 = chip8arc.clone();

    event_loop.run(move |ev, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
                render_texture_to_target(&eventloopchip8.read().unwrap().display, pixels.get_frame(), &flags.fg, &flags.bg);
                pixels.render().unwrap();
            }
            Event::WindowEvent { window_id: _, event: window_ev } => match window_ev {
                WindowEvent::KeyboardInput {input, device_id: _, is_synthetic: _ } => {
                    parse_input(input, &mut eventloopchip8.write().unwrap(), &flags);
                }
                WindowEvent::Resized(size) => {
                    pixels.resize_surface(size.width, size.height);
                }
                _ => ()
            },
            _ => (),
        }
        window.request_redraw()
    });
}

fn render_texture_to_target(dispmem: &[u8; 2048], frame: &mut [u8], fg: &Rgb, bg: &Rgb) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        if dispmem[i] == 1 {
            pixel.copy_from_slice(&[fg.r, fg.g, fg.b, 0xff]);
        }
        else {
            pixel.copy_from_slice(&[bg.r, bg.g, bg.b, 0xff]);
        }
    }
}