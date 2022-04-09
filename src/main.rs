mod chip8;
mod opcode_parser;
mod fstools;

use crate::fstools::get_file_as_byte_vec;
use crate::chip8::Chip8;

extern crate glium;

fn main() {
    let mut chip8inst = Chip8::new();
    chip8inst.load_program(&get_file_as_byte_vec("./roms/bo.ch8"));

    let mut runtimes = 0;

    use glium::{glutin, Surface};
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    event_loop.run(move |ev, _, control_flow| {
        // set next run time to 1 second / 500hz =  2 milliseconds.
        let next_frame_time = std::time::Instant::now() +std::time::Duration::from_millis(2);

        // timer stuff
        if runtimes >= 8 {
            if chip8inst.delay_timer > 0 {
                chip8inst.delay_timer -= 1;
            }
            if chip8inst.sound_timer > 0 {
                chip8inst.delay_timer -= 1;
            }
            runtimes = 0;
        }
        else {
            runtimes += 1;
        }

        // cycle cpu
        chip8inst.single_cycle();
        
        // render frame
        let mut disptexturevec = vec![vec![(0u8, 0u8, 0u8); 64]; 32];
        for (i, e) in  chip8inst.display.iter().enumerate() {
            if *e == 1 {
                disptexturevec[31 - (i % 32)][i / 32] = (255u8, 255u8, 255u8);
            }
        }
        let texture = glium::Texture2d::new(&display, disptexturevec).unwrap();
        // get texture from display

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        texture.as_surface().fill(&target, glium::uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                    // println!("{:?}", input.virtual_keycode.unwrap());
                    chip8inst.keystate[0x1] = 1;
                },
                _ => return,
            },
            _ => (),
        }
    });
}