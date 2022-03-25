use lle_simulator::*;
use std::time::{Duration, Instant};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

const INPUT_BAR: &[&str] = &["alpha", "pump", "linear"];
const INPUT_STATIC: &[&str] = &["record_step", "simu_step"];

struct Control {
    desc: String,
    slide_bar: Option<(f64, f64, f64)>,
    current: f64,
}

impl Component for Control {
    type Message = ();

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        todo!()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        todo!()
    }
}

struct ControlProps {
    pub children: ChildrenWithProps<Control>,
}

struct App {
    worker: Worker,
    input: Vec<Control>,
    last_frame: Instant,
}

impl Component for App {
    type Message;

    type Properties;

    fn create(ctx: &Context<Self>) -> Self {
        todo!()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        todo!()
    }
}

fn main() {
    use console_error_panic_hook::set_once as set_panic_hook;
    use wasm_bindgen::prelude::*;
    use web_sys::window;
    set_panic_hook();
    log!("running main in rust");
}
