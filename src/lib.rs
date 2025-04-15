use std::f64::consts::PI;

pub use anyhow::{anyhow, Result};
use lle::{num_complex::Complex64, CoupleOp, CoupledLleSolver, Evolver, LinearOp, LleSolver};

pub struct Worker {
    core: CoupledLleSolver<
        f64,
        [Complex64; SHELL_LEN],
        [Complex64; SHELL_LEN],
        lle::LinearOpAdd<f64, (lle::DiffOrder, Complex64), (lle::DiffOrder, Complex64)>,
        lle::LinearOpAdd<f64, (lle::DiffOrder, Complex64), (lle::DiffOrder, Complex64)>,
        lle::SPhaMod,
        lle::SPhaMod,
        Complex64,
        lle::NoneOp<f64>,
        lle::NoneOp<f64>,
        lle::NoneOp<f64>,
        lle::CoupleOpWithNonLinear<lle::ModeSplit<f64>, lle::XPhaMod>,
    >,
    property: WorkerProperty,
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
    RecordStep(u32),
    SimuStep(f64),
    Couple(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkerProperty {
    pub alpha: f64,
    pub pump: f64,
    pub linear: f64,
    pub record_step: u32,
    pub simu_step: f64,
    pub couple: f64,
}

const SHELL_LEN: usize = 128;

impl Worker {
    pub fn new() -> Self {
        const STEP_DIST: f64 = 8e-4;
        const PUMP: f64 = 3.94;
        const LINEAR: f64 = -0.0444;
        const ALPHA: f64 = -5.;
        const COUPLE: f64 = 1.;
        use rand::Rng;
        let mut rand = rand::rng();
        let mut init2 = [Complex64::new(0., 0.); SHELL_LEN];
        let mut init1 = init2.clone();
        init1.iter_mut().for_each(|x| {
            *x = (Complex64::i() * rand.random::<f64>() * 2. * PI).exp()
                * (-(rand.random::<f64>() * 1e5).powi(2)).exp()
        });
        init2.iter_mut().for_each(|x| {
            *x = (Complex64::i() * rand.random::<f64>() * 2. * PI).exp()
                * (-(rand.random::<f64>() * 1e5).powi(2)).exp()
        });
        let lle1 = LleSolver::builder()
            .state(init1)
            .step_dist(STEP_DIST)
            .constant(Complex64::from(PUMP))
            .linear(
                (0, -(Complex64::i() * ALPHA + 1.))
                    .add_linear_op((2, -Complex64::i() * LINEAR / 2.)),
            )
            .nonlin(lle::SPhaMod)
            .constant_freq(lle::NoneOp::default())
            .build();
        let lle2 = LleSolver::builder()
            .state(init2)
            .step_dist(STEP_DIST)
            .constant(lle::NoneOp::default())
            .linear(
                (0, -(Complex64::i() * ALPHA + 1.))
                    .add_linear_op((2, -Complex64::i() * LINEAR / 2.)),
            )
            .nonlin(lle::SPhaMod)
            .constant_freq(lle::NoneOp::default())
            .build();
        Worker {
            core: CoupledLleSolver::builder()
                .component1(lle1)
                .component2(lle2)
                .couple(
                    lle::ModeSplit::builder()
                        .strength(COUPLE)
                        .mode(0)
                        .build()
                        .with_nonlinear(lle::XPhaMod),
                )
                .build(),

            property: WorkerProperty {
                alpha: ALPHA,
                linear: LINEAR,
                pump: PUMP,
                record_step: 100,
                simu_step: STEP_DIST,
                couple: COUPLE,
            },
        }
    }
    pub fn get_property(&self) -> WorkerProperty {
        self.property
    }
    pub fn set_property(&mut self, update: WorkerUpdate) {
        match update {
            WorkerUpdate::Alpha(value) => {
                self.property.alpha = value;
                self.core.component1.linear = (0, -(Complex64::i() * self.property.alpha + 1.))
                    .add_linear_op((2, -Complex64::i() * self.property.linear / 2.))
                    .into();
                self.core.component2.linear = (0, -(Complex64::i() * self.property.alpha + 1.))
                    .add_linear_op((2, -Complex64::i() * self.property.linear / 2.))
                    .into()
            }
            WorkerUpdate::Pump(value) => {
                self.property.pump = value;
                self.core.component1.constant = Complex64::from(value).into();
            }
            WorkerUpdate::Linear(value) => {
                self.property.linear = value;
                self.core.component1.linear = (0, -(Complex64::i() * self.property.alpha + 1.))
                    .add_linear_op((2, -Complex64::i() * self.property.linear / 2.))
                    .into();
                self.core.component2.linear = (0, -(Complex64::i() * self.property.alpha + 1.))
                    .add_linear_op((2, -Complex64::i() * self.property.linear / 2.))
                    .into()
            }
            WorkerUpdate::RecordStep(value) => self.property.record_step = value as u32,
            WorkerUpdate::SimuStep(value) => {
                self.property.simu_step = value;
                self.core.component1.step_dist = value;
                self.core.component2.step_dist = value;
            }
            WorkerUpdate::Couple(value) => {
                self.property.couple = value;
                self.core.couple.couple.strength = value;
            }
        }
    }
    pub fn tick(&mut self) {
        use rand::Rng;
        let mut rand = rand::rng();
        self.core.component1.state_mut().iter_mut().for_each(|x| {
            *x += (Complex64::i() * rand.random::<f64>() * 2. * PI).exp()
                * (-(rand.random::<f64>() * 1e5).powi(2)).exp()
        });
        self.core.component2.state_mut().iter_mut().for_each(|x| {
            *x += (Complex64::i() * rand.random::<f64>() * 2. * PI).exp()
                * (-(rand.random::<f64>() * 1e5).powi(2)).exp()
        });
        log::info!("tick steps {}", self.property.simu_step);
        self.core.evolve_n(self.property.record_step);
        log::info!("tick finished");
    }
    pub fn get_state(&self) -> (&[Complex64], &[Complex64]) {
        (self.core.component1.state(), self.core.component2.state())
    }
}
