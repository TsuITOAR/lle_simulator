use plotters::prelude::DrawingBackend;

use crate::*;
const MAX_LEN: usize = 20;
impl<DB: DrawingBackend> super::Worker<DB>
where
    <DB as plotters::prelude::DrawingBackend>::ErrorType: 'static,
{
    pub(crate) fn draw(
        &mut self,
        s: Vec<(f64, f64)>,
    ) -> Result<impl Fn((i32, i32)) -> Option<(f64, f64)>> {
        self.history.push(s.iter().map(|x| x.1).collect());
        let _s = self.history.draw()?;
        Ok(self.animator.new_frame(s)?)
    }
}
