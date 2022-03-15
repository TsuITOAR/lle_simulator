use crate::DrawResult;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;

impl super::Worker {
    pub fn draw(&self, canvas_id: &str) -> DrawResult<impl Fn((i32, i32)) -> Option<(f32, f32)>> {
        let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
        let root = backend.into_drawing_area();
        root.fill(&WHITE)?;
        let com_float = |x: &&f64, y: &&f64| -> std::cmp::Ordering {
            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
        };
        let mut chart = ChartBuilder::on(&root)
            .margin(20i32)
            .x_label_area_size(30i32)
            .y_label_area_size(30i32)
            .build_cartesian_2d(
                0f32..self.state_len() as f32,
                (*self.shell.iter().min_by(com_float).unwrap_or(&0.) as f32).floor()
                    ..(*self.shell.iter().max_by(com_float).unwrap_or(&1.) as f32).ceil(),
            )?;

        chart.configure_mesh().x_labels(3).y_labels(3).draw()?;

        chart.draw_series(LineSeries::new(
            (0..self.state_len()).map(|x| (x as f32, self.shell[x] as f32)),
            &RED,
        ))?;

        root.present()?;
        return Ok(chart.into_coord_trans());
    }
}
