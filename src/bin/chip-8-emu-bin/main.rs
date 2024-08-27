mod fstools;
mod args;

use std::sync::{Arc, RwLock};

use chip_8_emu::{audio::Beeper, chip8::Chip8, input::parse_input, utils::render_texture_to_target};
use fstools::{get_file_as_byte_vec, load_state, save_state};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    // args
    let args = crate::args::parse_args();

    // setup speed
    // devide 2 as fetch and decode is on same loop
    let runhz: u64 = args.options.hz;
    let delay: u64 = 1000 / runhz;
    let satisfiedruntimes: u64 = (1000 / 60) /delay;

    // setup cpu instance
    let mut chip8inst = Chip8::default();
    chip8inst.display = [args.options.invert_colors as u8; 2048];

    // load rom/state into chip8inst
    let rompath = args.rom_path.as_str();
    if rompath.ends_with(".state") {
        crate::fstools::load_state(std::path::Path::new(rompath), &mut chip8inst)
    }
    else {
        chip8inst.load_program(&get_file_as_byte_vec(rompath));
    }
    let chip8arc = Arc::new(RwLock::new(chip8inst));

    let loopchip8 = chip8arc.clone();
    std::thread::spawn(move || {
        let beeper = Beeper::new(args.options.vol);
        let beeperexist = beeper.is_ok() && args.options.vol > 0.0;
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
                render_texture_to_target(&eventloopchip8.read().unwrap().display, pixels.frame_mut(), &args.options.fg, &args.options.bg);
                pixels.render().unwrap();
            }
            Event::WindowEvent { window_id: _, event: window_ev } => match window_ev {
                WindowEvent::KeyboardInput {input, device_id: _, is_synthetic: _ } => {
                    if let Some((key, pressed)) = parse_input(input) {
                        eventloopchip8.write().unwrap().keystate[key] = pressed as u8;
                    }
                    let pressed = (input.state == ElementState::Pressed) as u8;
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        match virtual_keycode {
                            VirtualKeyCode::F5 => {
                                if pressed == 1 {
                                    let rompath = std::path::Path::new(args.rom_path.as_str());
                                    let statepath = rompath.with_extension("state");
                
                                    save_state(&statepath, &eventloopchip8.read().unwrap());
                                }
                            },
                            VirtualKeyCode::F6 => {
                                if pressed == 1 {
                                    let rompath = std::path::Path::new(args.rom_path.as_str());
                                    let statepath = rompath.with_extension("state");
                
                                    load_state(&statepath, &mut eventloopchip8.write().unwrap())
                                }
                            },
                            _ => {}
                        }
                    }
                }
                WindowEvent::Resized(size) => {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
                _ => ()
            },
            _ => (),
        }
        window.request_redraw()
    });
}