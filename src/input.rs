use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub static KEYMAP: [usize; 16] = [
    0x1, // 1
    0x2, // 2
    0x3, // 3
    0xC, // C
    0x4, // 4
    0x5, // 5
    0x6, // 6
    0xD, // D
    0x7, // 7
    0x8, // 8
    0x9, // 9
    0xE, // E
    0xA, // A
    0x0, // 0
    0xB, // B
    0xF, // F
];

pub fn parse_input(input: KeyboardInput) -> Option<(usize, bool)> {
    let pressed = input.state == ElementState::Pressed;
    if let Some(virtual_keycode) = input.virtual_keycode {
        match virtual_keycode {
            VirtualKeyCode::Key1=> {
                return Some((KEYMAP[0], pressed));
            },
            VirtualKeyCode::Key2=> {
                return Some((KEYMAP[1], pressed));
            },
            VirtualKeyCode::Key3=> {
                return Some((KEYMAP[2], pressed));
            },
            VirtualKeyCode::Key4=> {
                return Some((KEYMAP[3], pressed));
            },

            VirtualKeyCode::Q=> {
                return Some((KEYMAP[4], pressed));
            },
            VirtualKeyCode::W=> {
                return Some((KEYMAP[5], pressed));
            },
            VirtualKeyCode::E=> {
                return Some((KEYMAP[6], pressed));
            },
            VirtualKeyCode::R=> {
                return Some((KEYMAP[7], pressed));
            },

            VirtualKeyCode::A=> {
                return Some((KEYMAP[8], pressed));
            },
            VirtualKeyCode::S=> {
                return Some((KEYMAP[9], pressed));
            },
            VirtualKeyCode::D=> {
                return Some((KEYMAP[10], pressed));
            },
            VirtualKeyCode::F=> {
                return Some((KEYMAP[11], pressed));
            },

            VirtualKeyCode::Z=> {
                return Some((KEYMAP[12], pressed));
            },
            VirtualKeyCode::X=> {
                return Some((KEYMAP[13], pressed));
            },
            VirtualKeyCode::C=> {
                return Some((KEYMAP[14], pressed));
            },
            VirtualKeyCode::V=> {
                return Some((KEYMAP[15], pressed));
            },
            _ => { }
        }
    }
    None
}