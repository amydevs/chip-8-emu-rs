
use wasm_bindgen::prelude::*;
use web_sys::Element;
use std::sync::{mpsc::{channel, Sender}, Arc, RwLock};

use crate::{audio::Beeper, chip8::Chip8, input::parse_input, options::Options, utils::render_texture_to_target};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop}, platform::web::EventLoopExtWebSys, window::WindowBuilder
};
use winit::platform::web::WindowExtWebSys;
use gloo_console::log;
use gloo_timers::future::TimeoutFuture;

#[wasm_bindgen(start)]
pub fn init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Trace).expect("error initializing logger");
}

enum MainLoopMessage {
    Stop,
}

#[wasm_bindgen]
pub struct Wasm {
    main_loop_tx: Sender<MainLoopMessage>,
    event_loop_tx: Sender<MainLoopMessage>,
}

#[wasm_bindgen]
impl Wasm {
    pub async fn create(parent: Element, rom: &[u8], options: Options) -> Self {
        // setup speed
        // devide 2 as fetch and decode is on same loop
        let run_hz:  u64 = options.hz;
        let delay: u64 = 1000/run_hz;
        let satisfied_run_times: u64 = (1000/60)/delay;
    
        // setup cpu instance
        let mut chip8_inst = Chip8::default();
        chip8_inst.display = [options.invert_colors; 2048];
    
        // load rom/state into chip8inst
        chip8_inst.load_program(rom);
    
        let chip8_arc = Arc::new(RwLock::new(chip8_inst));
    
        let main_loop_chip8 = chip8_arc.clone();
        let (main_loop_tx, main_loop_rx) = channel::<MainLoopMessage>();
        wasm_bindgen_futures::spawn_local(async move {
            let beeper = Beeper::new(options.vol);
            let beeperexist = beeper.is_ok() && options.vol > 0.0;
            if !beeperexist {
                log!("Audio not initialized!");
            }
    
            let mut run_times = 0;
            loop {
                if  let Ok(mesg) = main_loop_rx.try_recv() {
                    match mesg {
                        MainLoopMessage::Stop => break,
                    }
                }
                let next_frame_time = js_sys::Date::now() as u64 + delay;
                
                // timer stuff
                if run_times >= satisfied_run_times {
                    if main_loop_chip8.read().unwrap().delay_timer > 0 {
                        main_loop_chip8.write().unwrap().delay_timer -= 1;
                    }
                    if main_loop_chip8.read().unwrap().sound_timer > 0 {
                        if beeperexist {
                            beeper.as_ref().unwrap().play();
                        }
                        main_loop_chip8.write().unwrap().sound_timer -= 1;
                    }
                    else if beeperexist {
                        beeper.as_ref().unwrap().pause();
                    }
    
                    run_times = 0;
                }
                run_times += 1;
    
    
                // cycle cpu
                main_loop_chip8.write().unwrap().single_cycle();
    
                let final_date = js_sys::Date::now() as u64;
                if next_frame_time > final_date {
                    TimeoutFuture::new((next_frame_time - final_date) as u32).await;
                }
            }
        });
    
        // setup opengl
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let canvas_element = web_sys::Element::from(window.canvas());
        parent.append_child(&canvas_element).unwrap();
        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new_async(64, 32, surface_texture).await.unwrap()
        };
    
        let event_loop_chip8 = chip8_arc.clone();
        let (event_loop_tx, event_loop_rx) = channel::<MainLoopMessage>();
        wasm_bindgen_futures::spawn_local(async move {
            event_loop.spawn(move |ev, _, control_flow| {
                *control_flow = ControlFlow::Poll;


                if  let Ok(mesg) = event_loop_rx.try_recv() {
                    match mesg {
                        MainLoopMessage::Stop => {
                            *control_flow = ControlFlow::Exit;
                            canvas_element.remove();
                            return;
                        },
                    }
                }

                match ev {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                    Event::RedrawRequested(_) => {
                        render_texture_to_target(&event_loop_chip8.read().unwrap().display, pixels.frame_mut(), &options.fg, &options.bg);
                        pixels.render().unwrap();
                    }
                    Event::WindowEvent { window_id: _, event: window_ev } => match window_ev {
                        WindowEvent::KeyboardInput {input, device_id: _, is_synthetic: _ } => {
                            parse_input(input, &mut event_loop_chip8.write().unwrap());
                            let pressed = (input.state == ElementState::Pressed) as u8;
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                match virtual_keycode {
                                    VirtualKeyCode::F5 => {
                                        if pressed == 1 {
                                        }
                                    },
                                    VirtualKeyCode::F6 => {
                                        if pressed == 1 {
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
        });
        
        Self {
            event_loop_tx,
            main_loop_tx,
        }
    }
    pub fn stop(self) {
        self.event_loop_tx.send(MainLoopMessage::Stop).unwrap();
        self.main_loop_tx.send(MainLoopMessage::Stop).unwrap();
        drop(self);
    }
}