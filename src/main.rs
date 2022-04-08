mod chip8;
mod opcodes;
mod fstools;

use glium::texture;

use crate::fstools::get_file_as_byte_vec;
use crate::chip8::Chip8;

extern crate glium;

fn main() {
    let mut chip8inst = Chip8::new();
    chip8inst.load_program(&get_file_as_byte_vec("./roms/ibm.ch8"));

    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    event_loop.run(move |ev, _, control_flow| {
        chip8inst.single_cycle();
        let texture = glium::Texture2d::new(&display,
            vec![
            vec![(0u8, 1u8, 2u8), (4u8, 8u8, 16u8), (4u8, 8u8, 16u8), (4u8, 8u8, 16u8)]
        ]).unwrap();

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        texture.as_surface().fill(&target, glium::uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();
        
        // let next_frame_time = std::time::Instant::now() +
        //     std::time::Duration::from_nanos(16_666_667);

        // *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            _ => (),
        }
    });
}