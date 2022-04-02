mod plot;
mod utils;

use std::f64::consts::PI;

pub use anyhow::{anyhow, Result};
use jkplot::{Animator, ColorMapVisualizer};
use lle::{num_complex::Complex64, Evolver, LinearOp, LleSolver};
use plotters::prelude::*;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct Worker<DB: DrawingBackend> {
    core: LleSolver<
        f64,
        [Complex64; SHELL_LEN],
        lle::LinearOpAdd<(lle::DiffOrder, Complex64), (lle::DiffOrder, Complex64)>,
    >,
    property: WorkerProperty,
    animator: Animator<DB>,
    history: ColorMapVisualizer<DB>,
}
pub struct CursorPos {
    convert: Box<dyn Fn((i32, i32)) -> Option<(f64, f64)>>,
}

impl CursorPos {
    pub fn coord(&self, x: i32, y: i32) -> Option<Point> {
        (self.convert)((x, y)).map(|(x, y)| Point { x, y })
    }
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum WorkerUpdate {
    Alpha(f64),
    Pump(f64),
    Linear(f64),
    RecordStep(u64),
    SimuStep(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkerProperty {
    pub alpha: f64,
    pub pump: f64,
    pub linear: f64,
    pub record_step: u64,
    pub simu_step: f64,
}

const SHELL_LEN: usize = 128;

impl<DB: DrawingBackend + 'static> Worker<DB> {
    pub fn new(line: DB, map: DB) -> Self {
        const STEP_DIST: f64 = 8e-4;
        const PUMP: f64 = 3.94;
        const LINEAR: f64 = -0.0444;
        const ALPHA: f64 = -5.;
        use rand::Rng;
        utils::set_panic_hook();
        let mut rand = rand::thread_rng();
        let mut init = [Complex64::new(0., 0.); SHELL_LEN];
        init.iter_mut().for_each(|x| {
            *x = (Complex64::i() * rand.gen::<f64>() * 2. * PI).exp()
                * (-(rand.gen::<f64>() * 1e5).powi(2)).exp()
        });
        let mut animator = Animator::on_backend(line);
        animator.set_min_y_range(1e-4);
        let history = ColorMapVisualizer::on_backend(map);
        Worker {
            core: LleSolver::new(
                [Complex64::new(0., 0.); SHELL_LEN],
                STEP_DIST,
                (0, -(Complex64::i() * ALPHA + 1.)).add((2, -Complex64::i() * LINEAR / 2.)),
                Box::new(|x: Complex64| Complex64::i() * x.norm_sqr())
                    as Box<dyn Fn(Complex64) -> Complex64>,
                Complex64::from(PUMP),
            ),
            property: WorkerProperty {
                alpha: ALPHA,
                linear: LINEAR,
                pump: PUMP,
                record_step: 100,
                simu_step: STEP_DIST,
            },
            animator,
            history,
        }
    }
    pub fn get_property(&self) -> WorkerProperty {
        self.property
    }
    pub fn set_property(&mut self, update: WorkerUpdate) {
        match update {
            WorkerUpdate::Alpha(value) => {
                self.property.alpha = value;
                self.core.linear = (0, -(Complex64::i() * self.property.alpha + 1.))
                    .add((2, -Complex64::i() * self.property.linear / 2.))
                    .into()
            }
            WorkerUpdate::Pump(value) => {
                self.property.pump = value;
                self.core.constant = Complex64::from(value).into();
            }
            WorkerUpdate::Linear(value) => {
                self.property.linear = value;
                self.core.linear = (0, -(Complex64::i() * self.property.alpha + 1.))
                    .add((2, -Complex64::i() * self.property.linear / 2.))
                    .into()
            }
            WorkerUpdate::RecordStep(value) => self.property.record_step = value as u64,
            WorkerUpdate::SimuStep(value) => {
                self.property.simu_step = value;
                self.core.step_dist = value;
            }
        }
    }
    pub fn tick(&mut self) -> Result<CursorPos> {
        use rand::Rng;
        let mut rand = rand::thread_rng();
        self.core.state_mut().iter_mut().for_each(|x| {
            *x += (Complex64::i() * rand.gen::<f64>() * 2. * PI).exp()
                * (-(rand.gen::<f64>() * 1e5).powi(2)).exp()
        });
        self.core.evolve_n(self.property.record_step);
        let v: Vec<_> = self
            .core
            .state()
            .iter()
            .enumerate()
            .map(|(x, y)| (x as f64, y.re))
            .collect();
        let map_coord = self.draw(v)?;
        Ok(CursorPos {
            convert: Box::new(move |coord| map_coord(coord).map(|(x, y)| (x.into(), y.into()))),
        })
    }
}
