
use wasm_bindgen::prelude::*;
use web_sys::Element;
use std::sync::{mpsc::{channel, Sender}, Arc, Mutex, RwLock};

use crate::{audio::Beeper, chip8::Chip8, input::{parse_input, KEYMAP}, options::{Options, RGB}, utils::render_texture_to_target};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, platform::web::{EventLoopExtWebSys, WindowExtWebSys}, window::{Window, WindowBuilder}
};
use gloo_console::log;
use gloo_timers::future::TimeoutFuture;

#[wasm_bindgen(start)]
pub fn init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Trace).expect("error initializing logger");
}

struct WasmEventLoopOptions {
    fg: RGB,
    bg: RGB,
}

impl From<Options> for WasmEventLoopOptions {
    fn from(options: Options) -> Self {
        Self {
            bg: options.bg,
            fg: options.fg
        }
    }
}

enum WasmEventLoopMessage {
    Attach(WasmMainLoop),
    WasmMainLoopMessage(WasmMainLoopMessage),
}

#[wasm_bindgen]
pub struct WasmEventLoop {
    tx: Sender<WasmEventLoopMessage>,
    main_loop_wrapper: Arc<Mutex<Option<WasmMainLoopWrapper>>>
}

#[wasm_bindgen]
impl WasmEventLoop {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let (tx, rx) = channel::<WasmEventLoopMessage>();

        let main_loop_wrapper: Arc<Mutex<Option<WasmMainLoopWrapper>>> = Arc::new(Mutex::new(None));
        let inst = Self {
            tx,
            main_loop_wrapper: main_loop_wrapper,
        };

        let future_main_loop_wrapper = inst.main_loop_wrapper.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let event_loop = EventLoop::new();
            event_loop.spawn(move |ev, target, control_flow,| {
                *control_flow = ControlFlow::Poll;

                for mesg in rx.try_iter() {
                    match mesg {
                        WasmEventLoopMessage::Attach(main_loop) => {
                            if let Some(main_loop_wrapper) = future_main_loop_wrapper.lock().unwrap().as_ref() {
                                main_loop_wrapper.main_loop.stop();
                                main_loop_wrapper.window.canvas().remove();
                            }
                            let window = WindowBuilder::new().build(&target).unwrap();
                            window.set_inner_size(winit::dpi::PhysicalSize::new(640, 320));
                            main_loop.parent.append_child(&window.canvas()).unwrap();
                            let window_size = window.inner_size();
                            let pixels_main_loop_wrapper = future_main_loop_wrapper.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
                                let pixels = Pixels::new_async(64, 32, surface_texture).await.unwrap();
                                *pixels_main_loop_wrapper.lock().unwrap() = Some(WasmMainLoopWrapper {
                                    main_loop,
                                    pixels,
                                    window,
                                });
                            });
                        },
                        WasmEventLoopMessage::WasmMainLoopMessage(mesg) => {
                            match mesg {
                                WasmMainLoopMessage::Stop => {
                                    if let Some(main_loop_wrapper) = future_main_loop_wrapper.lock().unwrap().as_ref() {
                                        main_loop_wrapper.main_loop.tx.send(mesg).unwrap();
                                        main_loop_wrapper.window.canvas().remove();
                                    }
                                    *future_main_loop_wrapper.lock().unwrap() = None;
                                }
                                _ => {
                                    if let Some(main_loop_wrapper) = future_main_loop_wrapper.lock().unwrap().as_mut() {
                                        main_loop_wrapper.main_loop.tx.send(mesg).unwrap();
                                    }
                                }
                            }
                        }
                    };
                }

                if let Some(main_loop_wrapper) = future_main_loop_wrapper.lock().unwrap().as_mut() {
                    let main_loop = &mut main_loop_wrapper.main_loop;
                    let pixels = &mut main_loop_wrapper.pixels;
                    let window = &mut main_loop_wrapper.window;
                    match ev {
                        Event::RedrawRequested(_) => {
                            render_texture_to_target(&main_loop.chip8.read().unwrap().display, pixels.frame_mut(), &main_loop.event_loop_options.fg, &main_loop.event_loop_options.bg);
                            pixels.render().unwrap();
                        }
                        Event::WindowEvent { window_id: _, event: ref window_ev } => match window_ev {
                            WindowEvent::KeyboardInput {input, device_id: _, is_synthetic: _ } => {
                                if let Some((key, pressed)) = parse_input(*input) {
                                    main_loop.tx.send(WasmMainLoopMessage::SetKey(key, pressed)).unwrap();
                                }
                                // let pressed = (input.state == ElementState::Pressed) as u8;
                                // if let Some(virtual_keycode) = input.virtual_keycode {
                                //     match virtual_keycode {
                                //         VirtualKeyCode::F5 => {
                                //             if pressed == 1 {
                                //             }
                                //         },
                                //         VirtualKeyCode::F6 => {
                                //             if pressed == 1 {
                                //             }
                                //         },
                                //         _ => {}
                                //     }
                                // }
                            }
                            // WindowEvent::Resized(size) => {
                            //     pixels.resize_surface(size.width, size.height).unwrap();
                            // }
                            _ => ()
                        },
                        _ => (),
                    }
                    window.request_redraw()
                }
            });
        });
        inst
    }
    
    pub fn attach(&mut self, main_loop: WasmMainLoop) {
        self.tx.send(WasmEventLoopMessage::Attach(main_loop)).unwrap()
    }

    pub fn detach(&self) {
        self.tx.send(
            WasmEventLoopMessage::WasmMainLoopMessage(
                WasmMainLoopMessage::Stop
            )
        ).unwrap();
    }

    pub fn set_options(&mut self, options: Options) {
        if let Some(main_loop_wrapper) = self.main_loop_wrapper.lock().unwrap().as_mut() {
            main_loop_wrapper.main_loop.event_loop_options = WasmEventLoopOptions::from(options);
        }
        self.tx.send(
            WasmEventLoopMessage::WasmMainLoopMessage(
                WasmMainLoopMessage::SetOptions(WasmMainLoopOptions::from(options))
            )
        ).unwrap();
    }

    pub fn set_key(&self, key: u8, pressed: bool) {
        self.tx.send(
            WasmEventLoopMessage::WasmMainLoopMessage(
                WasmMainLoopMessage::SetKey(key as usize, pressed)
            )
        ).unwrap();
    }
}

// Main loop stuff

struct WasmMainLoopWrapper {
    main_loop: WasmMainLoop,
    pixels: Pixels,
    window: Window,
}

struct WasmMainLoopOptions {
    invert_colors: bool,
    hz: u64,
    vol: f32,
}

impl From<Options> for WasmMainLoopOptions {
    fn from(options: Options) -> Self {
        Self {
            invert_colors: options.invert_colors,
            hz: options.hz,
            vol: options.vol,
        }
    }
}

enum WasmMainLoopMessage {
    Stop,
    SetOptions(WasmMainLoopOptions),
    SetKey(usize, bool),
}

#[wasm_bindgen]
pub struct WasmMainLoop {
    tx: Sender<WasmMainLoopMessage>,
    chip8: Arc<RwLock<Chip8>>,
    parent: Element,
    event_loop_options: WasmEventLoopOptions,
}

#[wasm_bindgen]
impl WasmMainLoop {
    pub async fn create(parent: Element, rom: &[u8], options: Options) -> Self {
        let mut main_loop_options = WasmMainLoopOptions::from(options);
        // setup cpu instance
        let mut chip8_inst = Chip8::default();
        let mut invert_colors: bool = main_loop_options.invert_colors;
        chip8_inst.display = [invert_colors as u8; 2048];
    
        // load rom/state into chip8inst
        chip8_inst.load_program(rom);
    
        let chip8_arc = Arc::new(RwLock::new(chip8_inst));
    
        let main_loop_chip8 = chip8_arc.clone();
        let (tx, rx) = channel::<WasmMainLoopMessage>();
        wasm_bindgen_futures::spawn_local(async move {
            let mut beeper = Beeper::new(main_loop_options.vol);
            if !beeper.is_err() {
                log!("Audio not initialized!");
            }
    
            let mut run_times = 0;
            loop {
                let run_hz:  u64 = main_loop_options.hz;
                let delay: u64 = 1000 / run_hz;
                let next_frame_time = js_sys::Date::now() as u64 + delay;
                let satisfied_run_times: u64 = (1000 / 60) / delay;

                if let Ok(beeper) = beeper.as_mut() {
                    beeper.set_vol(main_loop_options.vol).unwrap();
                }
                if main_loop_options.invert_colors != invert_colors {
                    invert_colors = main_loop_options.invert_colors;
                    main_loop_chip8
                        .write()
                        .unwrap()
                        .display
                        .iter_mut()
                        .for_each(|x| {
                            if *x > 0 {
                                *x = 0;
                            }
                            else {
                                *x = 1;
                            }
                        });
                }

                for mesg in rx.try_iter() {
                    match mesg {
                        WasmMainLoopMessage::Stop => break,
                        WasmMainLoopMessage::SetOptions(mesg) => {
                            main_loop_options = mesg;
                        },
                        WasmMainLoopMessage::SetKey(key, pressed) => {
                            if !KEYMAP.contains(&(key as usize)) {
                                continue;
                            }
                            main_loop_chip8.write().unwrap().key_state[key] = pressed as u8;
                        
                        }
                    }
                }
                
                // timer stuff
                if run_times >= satisfied_run_times {
                    if main_loop_chip8.read().unwrap().delay_timer > 0 {
                        main_loop_chip8.write().unwrap().delay_timer -= 1;
                    }
                    if main_loop_chip8.read().unwrap().sound_timer > 0 {
                        if beeper.is_ok() && main_loop_options.vol > 0.0 {
                            beeper.as_ref().unwrap().play();
                        }
                        main_loop_chip8.write().unwrap().sound_timer -= 1;
                    }
                    else if beeper.is_ok() && main_loop_options.vol > 0.0 {
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

        Self {
            tx,
            chip8: chip8_arc,
            parent,
            event_loop_options: WasmEventLoopOptions::from(options),
        }
    }

    pub fn stop(&self) {
        self.tx.send(WasmMainLoopMessage::Stop).unwrap();
    }

    pub fn set_options(&mut self, options: Options) {
        self.event_loop_options = WasmEventLoopOptions::from(options);
        self.tx.send(WasmMainLoopMessage::SetOptions(WasmMainLoopOptions::from(options))).unwrap();
    }

    pub fn set_key(&self, key: u8, pressed: bool) {
        self.tx.send(
            WasmMainLoopMessage::SetKey(key as usize, pressed)
        ).unwrap();
    }
}
