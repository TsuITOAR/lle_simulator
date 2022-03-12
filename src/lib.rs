mod utils;

use lle::{num_complex::Complex64, Evolver, LinearOp, LleSolver};
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
    core: LleSolver<
        f64,
        [Complex64; 128],
        lle::LinearOpAdd<(lle::DiffOrder, Complex64), (lle::DiffOrder, Complex64)>,
    >,
    shell: [f64; 128],
    property: WorkerProperty,
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

#[wasm_bindgen]
impl Worker {
    pub fn new() -> Self {
        const STEP_DIST: f64 = 8e-4;
        const PUMP: f64 = 3.94;
        const LINEAR: f64 = -0.0444;
        const ALPHA: f64 = -5.;
        use lle::LinearOp;
        utils::set_panic_hook();
        Worker {
            core: LleSolver::new(
                [Complex64::new(1., 0.); 128],
                STEP_DIST,
                (0, -(Complex64::i() * ALPHA + 1.)).add((2, -Complex64::i() * LINEAR / 2.)),
                Box::new(|x: Complex64| Complex64::i() * x.norm_sqr())
                    as Box<dyn Fn(Complex64) -> Complex64>,
                Complex64::from(PUMP),
            ),
            shell: [1.; 128],
            property: WorkerProperty {
                alpha: ALPHA,
                linear: LINEAR,
                pump: PUMP,
                record_step: 100,
                simu_step: STEP_DIST,
            },
        }
    }
    pub fn get_property(&self) -> WorkerProperty {
        self.property
    }
    pub fn get_state(&self) -> *const f64 {
        self.shell.as_ptr()
    }
    pub fn get_len(&self) -> usize {
        self.shell.len()
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
    pub fn tick(&mut self) {
        log!("updating state in rust");
        self.core.evolve_n(self.property.record_step);
        self.shell
            .iter_mut()
            .zip(self.core.state().iter())
            .for_each(|(a, b)| *a = b.re);
        log!("updated state in rust");
    }
}
