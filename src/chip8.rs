use crate::opcode_parser::parse_op;

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

impl Chip8 {
    pub fn new() -> Self {
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
        chip8.load_fontset();
        chip8
    }
    fn load_fontset(&mut self) {
        for (i, font) in FONTSET.iter().enumerate() {
            self.memory[i] = *font;
        }
    }
    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[512..(program.len() + 512)].copy_from_slice(program);
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
        parse_op(self);
    }
}