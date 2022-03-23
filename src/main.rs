use lle_simulator::*;
use wasm_bindgen::prelude::*;

fn main() {
    use console_error_panic_hook::set_once as set_panic_hook;
    use wasm_bindgen::prelude::*;
    use web_sys::window;
    set_panic_hook();
    log!("running main in rust");
}
