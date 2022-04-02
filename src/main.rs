use anyhow::Result;
use lle_simulator::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};
fn main() -> Result<()> {
    Ok(())
}

enum SlideMessage {
    SetMax(f64),
    SetMin(f64),
    SetVal(f64),
}

struct LleSimulator {
    simulator: Worker,
}

struct MyChart;
impl Chart<Message> for MyChart {
    fn build_chart<DB: DrawingBackend>(&self, builder: ChartBuilder<DB>) {
        //build your chart here, please refer to plotters for more details
    }
}
