mod plot;
mod utils;

use std::f64::consts::PI;

use lle::{num_complex::Complex64, Evolver, LinearOp, LleSolver};
use wasm_bindgen::prelude::*;
pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;
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
    core: LleSolver<
        f64,
        [Complex64; SHELL_LEN],
        lle::LinearOpAdd<(lle::DiffOrder, Complex64), (lle::DiffOrder, Complex64)>,
    >,
    shell: [f64; SHELL_LEN],
    property: WorkerProperty,
    history: Vec<[f64; SHELL_LEN]>,
    history_ind: usize,
}

#[wasm_bindgen]
pub struct CursorPos {
    convert: Box<dyn Fn((i32, i32)) -> Option<(f64, f64)>>,
}

#[wasm_bindgen]
impl CursorPos {
    pub fn coord(&self, x: i32, y: i32) -> Option<Point> {
        (self.convert)((x, y)).map(|(x, y)| Point { x, y })
    }
}

#[wasm_bindgen]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WorkerProperty {
    pub alpha: f64,
    pub pump: f64,
    pub linear: f64,
    pub record_step: u64,
    pub simu_step: f64,
}

const SHELL_LEN: usize = 128;

#[wasm_bindgen]
impl Worker {
    pub fn new() -> Self {
        const STEP_DIST: f64 = 8e-4;
        const PUMP: f64 = 3.94;
        const LINEAR: f64 = -0.0444;
        const ALPHA: f64 = -5.;
        use lle::LinearOp;
        use rand::Rng;
        utils::set_panic_hook();
        let mut rand = rand::thread_rng();
        let mut init = [Complex64::new(0., 0.); SHELL_LEN];
        init.iter_mut().for_each(|x| {
            *x = (Complex64::i() * rand.gen::<f64>() * 2. * PI).exp()
                * (-(rand.gen::<f64>() * 1e5).powi(2)).exp()
        });
        let mut shell = [0.; SHELL_LEN];
        shell
            .iter_mut()
            .zip(init.iter())
            .for_each(|(a, b)| *a = b.re);
        Worker {
            core: LleSolver::new(
                [Complex64::new(0., 0.); SHELL_LEN],
                STEP_DIST,
                (0, -(Complex64::i() * ALPHA + 1.)).add((2, -Complex64::i() * LINEAR / 2.)),
                Box::new(|x: Complex64| Complex64::i() * x.norm_sqr())
                    as Box<dyn Fn(Complex64) -> Complex64>,
                Complex64::from(PUMP),
            ),
            shell: [0.; SHELL_LEN],
            property: WorkerProperty {
                alpha: ALPHA,
                linear: LINEAR,
                pump: PUMP,
                record_step: 100,
                simu_step: STEP_DIST,
            },
            history: vec![[0.; SHELL_LEN]; 100],
            history_ind: 0,
        }
    }
    pub fn get_property(&self) -> WorkerProperty {
        self.property
    }
    pub fn get_state(&self) -> *const f64 {
        self.shell.as_ptr()
    }
    pub fn state_len(&self) -> usize {
        self.shell.len()
    }
    pub fn history_len(&self) -> usize {
        self.history.len().max(1)
    }
    pub fn get_history(&self) -> *const f64 {
        self.history
            .first()
            .map_or(self.shell.as_ptr(), |x| x.as_ptr())
    }
    pub fn set_history_len(&mut self, new_len: usize) {
        self.history.resize(new_len, [0.; SHELL_LEN])
    }
    pub fn set_property(&mut self, property: String, value: f64) {
        match property.as_str() {
            "alpha" => {
                self.property.alpha = value;
                self.core.linear = (0, -(Complex64::i() * self.property.alpha + 1.))
                    .add((2, -Complex64::i() * self.property.linear / 2.))
                    .into()
            }
            "pump" => {
                self.property.alpha = value;
                self.core.constant = Complex64::from(value).into();
            }
            "linear" => {
                self.property.linear = value;
                self.core.linear = (0, -(Complex64::i() * self.property.alpha + 1.))
                    .add((2, -Complex64::i() * self.property.linear / 2.))
                    .into()
            }
            "record_step" => {
                self.property.record_step = value as u64;
            }
            "simu_step" => {
                self.property.simu_step = value;
            }
            s => {
                log!("Unknown property {}", s);
            }
        }
    }
    pub fn tick(&mut self, canvas_id: &str) -> Result<CursorPos, JsValue> {
        use rand::Rng;
        let mut rand = rand::thread_rng();
        self.core.state_mut().iter_mut().for_each(|x| {
            *x += (Complex64::i() * rand.gen::<f64>() * 2. * PI).exp()
                * (-(rand.gen::<f64>() * 1e5).powi(2)).exp()
        });
        self.core.evolve_n(self.property.record_step);
        self.shell
            .iter_mut()
            .zip(self.core.state().iter())
            .for_each(|(a, b)| *a = b.re);
        self.history[self.history_ind] = self.shell;
        self.history_ind = (self.history_ind + 1) % self.history.len();
        let map_coord = self.draw(canvas_id).map_err(|err| err.to_string())?;
        Ok(CursorPos {
            convert: Box::new(move |coord| map_coord(coord).map(|(x, y)| (x.into(), y.into()))),
        })
    }
}
