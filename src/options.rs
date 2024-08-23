use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

#[derive(Clone, Copy, Tsify, Serialize, Deserialize)]
#[serde(default)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Options {
    pub invert_colors: u8,
    pub hz: u64,
    pub fg: RGB,
    pub bg: RGB,
    pub vol: f32
}

impl Default for Options {
    fn default() -> Self {
        Self { 
            invert_colors: 0,
            hz: 500,
            fg: RGB {
                r: 255,
                g: 255,
                b: 255,
            },
            bg: RGB {
                r: 0,
                g: 0,
                b: 0,
            },
            vol: 1.0
        }
    }
}

#[derive(Clone, Copy, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8
}