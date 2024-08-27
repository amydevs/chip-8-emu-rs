pub mod chip8;
pub mod errors;
pub mod input;
pub mod audio;
pub mod options;
pub mod utils;
#[cfg(target_arch = "wasm32")]
pub mod wasm;