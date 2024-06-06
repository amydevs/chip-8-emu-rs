use crate::chip8::Chip8;
use rand::Rng;

pub fn parse_op(chip8: &mut Chip8) {
    let x = ((chip8.opcode & 0x0F00) >> 8) as usize;
    let y = ((chip8.opcode & 0x00F0) >> 4) as usize;
    let nn = (chip8.opcode & 0x00FF) as u8;

    // println!("{:X}", chip8.opcode);
    match chip8.opcode  {
        0x00E0 => {
            chip8.display = [0; 2048];
            return;
        },
        0x00EE => {
            // sets pc to the address at the top of the stack
            chip8.pc = chip8.jumpstack[chip8.stackpointer as usize];
            chip8.stackpointer -= 1;
            return;
        },
        _ => {}
    }

    // matches opcode with last 3 nibbles removed (ie, 0xA22A -> 0xA000)
    match chip8.opcode & 0xF000 {
        0x0000 => {
            // ignored by modern interpreters
            return;
        },
        0x1000 => {
            // 1NNN - jump to address NNN
            chip8.pc = chip8.opcode & 0x0FFF;
            return;
        }
        0x2000 => {
            // 2NNN - call subroutine at NNN
            chip8.stackpointer += 1;
            chip8.jumpstack[chip8.stackpointer as usize] = chip8.pc;
            chip8.pc = chip8.opcode & 0x0FFF;
            return;
        }
        0x3000 => {
            // 3XNN - skip next instruction if VX == NN
            if chip8.vregisters[x] == nn {
                chip8.pc += 2;
            }
        }
        0x4000 => {
            // 4XNN - skip next instruction if VX != NN
            if chip8.vregisters[x] != nn {
                chip8.pc += 2;
            }
            return;
        }
        0x6000 => {
            // 6XNN - set VX to NN
            chip8.vregisters[x] = nn;
            return;
        }
        0x7000 => {
            // 7XNN - add NN to VX
            chip8.vregisters[x] = (chip8.vregisters[x] as u16 + nn as u16) as u8;
            return;
        },
        0xA000 => {
            // ANNN - set I to NNN
            chip8.i = chip8.opcode & 0x0FFF;
            return;
        },
        0xB000 => {
            // BNNN - jump to address NNN + V0
            chip8.pc = (chip8.opcode & 0x0FFF) + chip8.vregisters[0] as u16;
            return;
        },
        0xC000 => {
            // CXNN - set VX to random byte ANDed with NN
            chip8.vregisters[x] = nn & rand::thread_rng().gen::<u8>();
            return;
        },
        0xD000 => {
            // DXYN - draw sprite at VX, VY with N bytes of sprite data starting at I
            let width = 8;
            let nbytes = chip8.opcode & 0x000F;

            // vregisters at x and y
            let vx = chip8.vregisters[x];
            let vy = chip8.vregisters[y];

            // set last register to 0
            chip8.vregisters[0xF] = 0;

            for row in 0..nbytes {
                // get the sprite from memory
                let mut sprt = chip8.memory[(chip8.i + row) as usize];

                for col in 0..width {
                    // if the sprite is not 0
                    if sprt & 0x0080 > 0 {
                        let disppixel = &mut chip8.display[(
                            ((vy as u16 + row) % 32) * 64 +
                            (vx as u16 + col) % 64
                        ) as usize];

                        // set last register to 1 if pixel is set
                        if *disppixel == 1 {
                            chip8.vregisters[0xF] = 1;
                        }

                        // toggle pixel
                        *disppixel ^= 1;
                    }

                    // shift the sprite to the right to be ready for next draw
                    sprt <<= 1;
                }
            }
            return;
        },
        _ => {}
    }

    // matches first and last nibbles (ie, 0xA22A -> 0xA00A) (mostly alu v-register operations)
    match chip8.opcode & 0xF00F {
        0x5000 => {
            // 5XY0 - skip next instruction if VX == VY
            if chip8.vregisters[x] == chip8.vregisters[y] {
                chip8.pc += 2;
            }
            return;
        },
        0x8000 => {
            // 8XY0 - set VX to VY
            chip8.vregisters[x] = chip8.vregisters[y];
            return;
        },
        0x8001 => {
            // 8XY1 - set VX to VX | VY
            chip8.vregisters[x] |= chip8.vregisters[y];
            return;
        },
        0x8002 => {
            // 8XY2 - set VX to VX & VY
            chip8.vregisters[x] &= chip8.vregisters[y];
            return;
        },
        0x8003 => {
            // 8XY3 - set VX to VX ^ VY
            chip8.vregisters[x] ^= chip8.vregisters[y];
            return;
        },
        0x8004 => {
            // 8XY4 - set VF to 1 if carry, set VX to VX + VY

            // Checks if the hex nibbles plussed together uses more than 8 bits, meaning it has carried over.
            let result = chip8.vregisters[x] as u16 + chip8.vregisters[y] as u16;
            if result > 0x00FF {
                chip8.vregisters[0xF] = 1;
            }
            else {
                chip8.vregisters[0xF] = 0;
            }
            chip8.vregisters[x] = chip8.vregisters[x].wrapping_add(chip8.vregisters[y]);
            return;
        },
        0x8005 => {
            // 8XY5 - set VF to 0 if borrow, set VX to VX - VY
            if chip8.vregisters[x] > chip8.vregisters[y] {
                chip8.vregisters[0xF] = 1;
            }
            else {
                chip8.vregisters[0xF] = 0;
            }

            chip8.vregisters[x] = chip8.vregisters[x].wrapping_sub(chip8.vregisters[y]);
            return;
        },
        0x8006 => {
            // 8XY6 - set VF to LSB of VX, set VX to VX >> 1

            // Set VF to least significant bit of VX
            chip8.vregisters[0xF] = chip8.vregisters[x] & 0x01;

            chip8.vregisters[x] >>= 1;

            return;
        },
        0x8007 => {
            // 8XY7 - set VX to VY - VX, set VF to 0 if borrow
            if chip8.vregisters[y] > chip8.vregisters[x] {
                chip8.vregisters[0xF] = 1;
            }
            else {
                chip8.vregisters[0xF] = 0;
            }

            chip8.vregisters[x] -= chip8.vregisters[y];
            return;
        },
        0x800E => {
            // 8XYE - set VX to VX << 1, set VF to MSB of VX

            // set registers by pushing unneeded bits off, and leaving with the MSB
            chip8.vregisters[0xF] = chip8.vregisters[x] >> 7;

            chip8.vregisters[x] <<= 1;
            return;
        },
        0x9000 => {
            // 9XY0 - skip next instruction if VX != VY
            if chip8.vregisters[x] != chip8.vregisters[y] {
                chip8.pc += 2;
            }
            return;
        },
        _ => {}
    }

    match chip8.opcode & 0xF0FF {
        0xE09E => {
            // EX9E - skip next instruction if key in VX is pressed
            if chip8.keystate[chip8.vregisters[x] as usize] != 0 {
                chip8.pc += 2;
            }
        },
        0xE0A1 => {
            // EXA1 - skip next instruction if key in VX is not pressed
            if chip8.keystate[chip8.vregisters[x] as usize] == 0 {
                chip8.pc += 2;
            }
        },
        0xF007 => {
            // FX07 - set VX to delay timer value
            chip8.vregisters[x] = chip8.delay_timer;
        },
        0xF00A => {
            // FX0A - wait for keypress, store in VX
            // todo: maybe broken!
            match chip8.keystate.iter().position(|&x| x != 0) {
                Some(key) => {
                    chip8.vregisters[x] = key as u8;
                },
                None => {
                    chip8.pc -= 2;
                }
            }
        },
        0xF015 => {
            // FX15 - set delay timer to VX
            chip8.delay_timer = chip8.vregisters[x];
        },
        0xF018 => {
            // FX18 - set sound timer to VX
            chip8.sound_timer = chip8.vregisters[x];
        },
        0xF01E => {
            // FX1E - add VX to I, set to I
            chip8.i += chip8.vregisters[x] as u16;
        },
        0xF029 => {
            // FX29 - set I to location of sprite for digit VX
            // multiplied by 5, as each sprite is 5 bytes long
            chip8.i = chip8.vregisters[x] as u16 * 5;
        },
        0xF033 => {
            // FX33 - store BCD representation of VX in memory locations I, I+1, and I+2
            chip8.memory[chip8.i as usize] = (chip8.vregisters[x] / 100) % 10;
            chip8.memory[(chip8.i + 1) as usize] = (chip8.vregisters[x] / 10) % 10;
            chip8.memory[(chip8.i + 2) as usize] = chip8.vregisters[x] % 10;
        },
        0xF055 => {
            // FX55 - store V0 to VX in memory starting at address I
            for index in 0..x {
                chip8.memory[chip8.i as usize + index] = chip8.vregisters[index];
            }
        },
        0xF065 => {
            // FX65 - read V0 to VX from memory starting at address I
            for index in 0..x {
                chip8.vregisters[index] = chip8.memory[chip8.i as usize + index];
            }
        },
        _ => {}
    }
}