mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(unused)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-lle!");
}

#[wasm_bindgen]
pub struct Worker {
    property: Vec<f64>,
}

#[wasm_bindgen]
impl Worker {
    pub fn new() -> Self {
        Self {
            property: (0..128).map(|x| x as f64).collect(),
        }
    }
    pub fn get_property(&self) -> *const f64 {
        self.property.as_ptr()
    }
    pub fn get_len(&self) -> usize {
        self.property.len()
    }
}
