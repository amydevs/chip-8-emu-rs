use std::io::{Read, Write};

use savefile::{load, save, SavefileError};
use savefile_derive::Savefile;
use rand::Rng;


// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM

static FONTSET: [u8; 80] = [
	0xF0, 0x90, 0x90, 0x90, 0xF0,		// 0
	0x20, 0x60, 0x20, 0x20, 0x70,		// 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0,		// 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0,		// 3
	0x90, 0x90, 0xF0, 0x10, 0x10,		// 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0,		// 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0,		// 6
	0xF0, 0x10, 0x20, 0x40, 0x40,		// 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0,		// 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0,		// 9
	0xF0, 0x90, 0xF0, 0x90, 0x90,		// A
	0xE0, 0x90, 0xE0, 0x90, 0xE0,		// B
	0xF0, 0x80, 0x80, 0x80, 0xF0,		// C
	0xE0, 0x90, 0x90, 0x90, 0xE0,		// D
	0xF0, 0x80, 0xF0, 0x80, 0xF0,		// E
	0xF0, 0x80, 0xF0, 0x80, 0x80		// F
];

#[derive(Savefile)]
pub struct Chip8 {
    
    // current opcode
    pub opcode: u16,
    pub memory: [u8; 4096],

    // V registers
    pub vregisters: [u8; 16],

    // index register and program counter (pc)
    pub i: u16,
    pub pc: u16,

    // interupts and hardware registers
    pub delay_timer: u8,
    pub sound_timer: u8,

    // stack used to remember the current location before a jump is performed.
    pub jumpstack: [u16; 16],
    // system has 16 levels of stack, to remember which level, a pointer is used.
    pub stackpointer: u16,

    // hex based keypad 0x0-0xF
    pub keystate: [u8; 16],

    pub display: [u8; 2048],
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut chip8 = Chip8 {
            opcode: 0,
            memory: [0; 4096],
            vregisters: [0; 16],
            i: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            jumpstack: [0; 16],
            stackpointer: 0,
            keystate: [0; 16],
            display: [0; 2048]
        };
        chip8.load_font_set(FONTSET);
        chip8
    }
}

impl Chip8 {
    pub fn load_font_set(&mut self, fontset: [u8; 80]) {
        for (i, font) in fontset.iter().enumerate() {
            self.memory[i] = *font;
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[512..(program.len() + 512)].copy_from_slice(program);
    }

    pub fn save_state(&mut self, writer: &mut dyn Write) -> Result<(), SavefileError> {
        save(writer, 1, self)
    }

    pub fn load_state(&mut self, reader: &mut dyn Read) -> Result<(), SavefileError> {
        let chip8 = load::<Self>(reader, 1)?;
        *self = chip8;
        Ok(())
    }

    pub fn single_cycle(&mut self) {
        // fetch
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);
        self.pc += 2;
        // decode - none
        // execute
        self.execute();
        // store
    }

    fn execute(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        let y = ((self.opcode & 0x00F0) >> 4) as usize;
        let nn = (self.opcode & 0x00FF) as u8;

        // println!("{:X}", self.opcode);
        match self.opcode  {
            0x00E0 => {
                self.display = [0; 2048];
                return;
            },
            0x00EE => {
                // sets pc to the address at the top of the stack
                self.pc = self.jumpstack[self.stackpointer as usize];
                self.stackpointer -= 1;
                return;
            },
            _ => {}
        }

        // matches opcode with last 3 nibbles removed (ie, 0xA22A -> 0xA000)
        match self.opcode & 0xF000 {
            0x0000 => {
                // ignored by modern interpreters
                return;
            },
            0x1000 => {
                // 1NNN - jump to address NNN
                self.pc = self.opcode & 0x0FFF;
                return;
            }
            0x2000 => {
                // 2NNN - call subroutine at NNN
                self.stackpointer += 1;
                self.jumpstack[self.stackpointer as usize] = self.pc;
                self.pc = self.opcode & 0x0FFF;
                return;
            }
            0x3000 => {
                // 3XNN - skip next instruction if VX == NN
                if self.vregisters[x] == nn {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // 4XNN - skip next instruction if VX != NN
                if self.vregisters[x] != nn {
                    self.pc += 2;
                }
                return;
            }
            0x6000 => {
                // 6XNN - set VX to NN
                self.vregisters[x] = nn;
                return;
            }
            0x7000 => {
                // 7XNN - add NN to VX
                self.vregisters[x] = (self.vregisters[x] as u16 + nn as u16) as u8;
                return;
            },
            0xA000 => {
                // ANNN - set I to NNN
                self.i = self.opcode & 0x0FFF;
                return;
            },
            0xB000 => {
                // BNNN - jump to address NNN + V0
                self.pc = (self.opcode & 0x0FFF) + self.vregisters[0] as u16;
                return;
            },
            0xC000 => {
                // CXNN - set VX to random byte ANDed with NN
                self.vregisters[x] = nn & rand::thread_rng().gen::<u8>();
                return;
            },
            0xD000 => {
                // DXYN - draw sprite at VX, VY with N bytes of sprite data starting at I
                let width = 8;
                let nbytes = self.opcode & 0x000F;

                // vregisters at x and y
                let vx = self.vregisters[x];
                let vy = self.vregisters[y];

                // set last register to 0
                self.vregisters[0xF] = 0;

                for row in 0..nbytes {
                    // get the sprite from memory
                    let mut sprt = self.memory[(self.i + row) as usize];

                    for col in 0..width {
                        // if the sprite is not 0
                        if sprt & 0x0080 > 0 {
                            let disppixel = &mut self.display[(
                                ((vy as u16 + row) % 32) * 64 +
                                (vx as u16 + col) % 64
                            ) as usize];

                            // set last register to 1 if pixel is set
                            if *disppixel == 1 {
                                self.vregisters[0xF] = 1;
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
        match self.opcode & 0xF00F {
            0x5000 => {
                // 5XY0 - skip next instruction if VX == VY
                if self.vregisters[x] == self.vregisters[y] {
                    self.pc += 2;
                }
                return;
            },
            0x8000 => {
                // 8XY0 - set VX to VY
                self.vregisters[x] = self.vregisters[y];
                return;
            },
            0x8001 => {
                // 8XY1 - set VX to VX | VY
                self.vregisters[x] |= self.vregisters[y];
                return;
            },
            0x8002 => {
                // 8XY2 - set VX to VX & VY
                self.vregisters[x] &= self.vregisters[y];
                return;
            },
            0x8003 => {
                // 8XY3 - set VX to VX ^ VY
                self.vregisters[x] ^= self.vregisters[y];
                return;
            },
            0x8004 => {
                // 8XY4 - set VF to 1 if carry, set VX to VX + VY

                // Checks if the hex nibbles plussed together uses more than 8 bits, meaning it has carried over.
                let result = self.vregisters[x] as u16 + self.vregisters[y] as u16;
                if result > 0x00FF {
                    self.vregisters[0xF] = 1;
                }
                else {
                    self.vregisters[0xF] = 0;
                }
                self.vregisters[x] = self.vregisters[x].wrapping_add(self.vregisters[y]);
                return;
            },
            0x8005 => {
                // 8XY5 - set VF to 0 if borrow, set VX to VX - VY
                if self.vregisters[x] > self.vregisters[y] {
                    self.vregisters[0xF] = 1;
                }
                else {
                    self.vregisters[0xF] = 0;
                }

                self.vregisters[x] = self.vregisters[x].wrapping_sub(self.vregisters[y]);
                return;
            },
            0x8006 => {
                // 8XY6 - set VF to LSB of VX, set VX to VX >> 1

                // Set VF to least significant bit of VX
                self.vregisters[0xF] = self.vregisters[x] & 0x01;

                self.vregisters[x] >>= 1;

                return;
            },
            0x8007 => {
                // 8XY7 - set VX to VY - VX, set VF to 0 if borrow
                if self.vregisters[y] > self.vregisters[x] {
                    self.vregisters[0xF] = 1;
                }
                else {
                    self.vregisters[0xF] = 0;
                }

                self.vregisters[x] -= self.vregisters[y];
                return;
            },
            0x800E => {
                // 8XYE - set VX to VX << 1, set VF to MSB of VX

                // set registers by pushing unneeded bits off, and leaving with the MSB
                self.vregisters[0xF] = self.vregisters[x] >> 7;

                self.vregisters[x] <<= 1;
                return;
            },
            0x9000 => {
                // 9XY0 - skip next instruction if VX != VY
                if self.vregisters[x] != self.vregisters[y] {
                    self.pc += 2;
                }
                return;
            },
            _ => {}
        }

        match self.opcode & 0xF0FF {
            0xE09E => {
                // EX9E - skip next instruction if key in VX is pressed
                if self.keystate[self.vregisters[x] as usize] != 0 {
                    self.pc += 2;
                }
            },
            0xE0A1 => {
                // EXA1 - skip next instruction if key in VX is not pressed
                if self.keystate[self.vregisters[x] as usize] == 0 {
                    self.pc += 2;
                }
            },
            0xF007 => {
                // FX07 - set VX to delay timer value
                self.vregisters[x] = self.delay_timer;
            },
            0xF00A => {
                // FX0A - wait for keypress, store in VX
                // todo: maybe broken!
                match self.keystate.iter().position(|&x| x != 0) {
                    Some(key) => {
                        self.vregisters[x] = key as u8;
                    },
                    None => {
                        self.pc -= 2;
                    }
                }
            },
            0xF015 => {
                // FX15 - set delay timer to VX
                self.delay_timer = self.vregisters[x];
            },
            0xF018 => {
                // FX18 - set sound timer to VX
                self.sound_timer = self.vregisters[x];
            },
            0xF01E => {
                // FX1E - add VX to I, set to I
                self.i += self.vregisters[x] as u16;
            },
            0xF029 => {
                // FX29 - set I to location of sprite for digit VX
                // multiplied by 5, as each sprite is 5 bytes long
                self.i = self.vregisters[x] as u16 * 5;
            },
            0xF033 => {
                // FX33 - store BCD representation of VX in memory locations I, I+1, and I+2
                self.memory[self.i as usize] = (self.vregisters[x] / 100) % 10;
                self.memory[(self.i + 1) as usize] = (self.vregisters[x] / 10) % 10;
                self.memory[(self.i + 2) as usize] = self.vregisters[x] % 10;
            },
            0xF055 => {
                // FX55 - store V0 to VX in memory starting at address I
                for index in 0..x {
                    self.memory[self.i as usize + index] = self.vregisters[index];
                }
            },
            0xF065 => {
                // FX65 - read V0 to VX from memory starting at address I
                for index in 0..x {
                    self.vregisters[index] = self.memory[self.i as usize + index];
                }
            },
            _ => {}
        }
    }
}