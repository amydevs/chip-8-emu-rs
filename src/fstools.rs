use std::path::Path;
use std::fs::{File, metadata};
use std::io::Read;
use std::path::{PathBuf};

use savefile::{save_file, load_file};

use crate::chip8::Chip8;

pub fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
    let mut f = File::open(filename).expect("no file found");
    let metadata = metadata(filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    buffer
}

pub fn save_state(filename: &Path, chip8inst: &Chip8) {
    save_file(filename, 0, chip8inst).unwrap_or_else(|x| {
        println!("{}", x);
    });
    println!("State saved: {}", filename.to_str().unwrap());
}

pub fn load_state(filename: &Path, chip8inst: &mut Chip8) {
    if filename.exists() {
        match load_file::<Chip8, PathBuf>(filename.to_path_buf(), 0) {
            Ok(state) => {
                *chip8inst = state;
            },
            Err(x) => {
                println!("{}", x);
            }
        }
        println!("State loaded: {}", filename.to_str().unwrap());
    }
    else {
        println!("No state file found!");
    }
}