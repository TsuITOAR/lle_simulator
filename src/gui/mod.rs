use super::*;

use lle_simulator::{WorkerProperty, WorkerUpdate};
#[allow(unused)]
use log::{debug, error, info, log_enabled, warn, Level};

mod chart;
mod message;
pub use chart::*;
pub use message::*;

fn from_property(p: &WorkerProperty, idx: usize) -> WorkerUpdate {
    match idx {
        0 => WorkerUpdate::Alpha(p.alpha),
        1 => WorkerUpdate::Pump(p.pump),
        2 => WorkerUpdate::Linear(p.linear),
        3 => WorkerUpdate::RecordStep(p.record_step),
        4 => WorkerUpdate::SimuStep(p.simu_step),
        5 => WorkerUpdate::Couple(p.couple),
        _ => unreachable!(),
    }
}

pub fn from_property_array(p: WorkerProperty) -> [WorkerUpdate; 6] {
    let mut a = [WorkerUpdate::Alpha(0.); 6];
    (0..6).into_iter().for_each(|x| a[x] = from_property(&p, x));
    a
}

pub fn property_value_to_string(v: WorkerUpdate) -> String {
    match v {
        WorkerUpdate::Alpha(v)
        | WorkerUpdate::Pump(v)
        | WorkerUpdate::Linear(v)
        | WorkerUpdate::SimuStep(v)
        | WorkerUpdate::Couple(v) => format!("{:.3E}", v),
        WorkerUpdate::RecordStep(v) => v.to_string(),
    }
}

pub fn map_property_to_idx(p: WorkerUpdate) -> usize {
    match p {
        WorkerUpdate::Alpha(_) => 0,
        WorkerUpdate::Pump(_) => 1,
        WorkerUpdate::Linear(_) => 2,
        WorkerUpdate::RecordStep(_) => 3,
        WorkerUpdate::SimuStep(_) => 4,
        WorkerUpdate::Couple(_) => 5,
    }
}

//only used from updating lower & higher bound of slider
pub fn extract_property_value(p: WorkerUpdate) -> f64 {
    match p {
        WorkerUpdate::Alpha(v) => v,
        WorkerUpdate::Pump(v) => v,
        WorkerUpdate::Linear(v) => v,
        WorkerUpdate::RecordStep(_) => unreachable!(),
        WorkerUpdate::SimuStep(_) => unreachable!(),
        WorkerUpdate::Couple(v) => v,
    }
}
