use crate::DrawResult;
impl super::Worker {
    pub(crate) fn draw(
        &mut self,
        s: impl IntoIterator<Item = (f64, f64)>,
    ) -> DrawResult<impl Fn((i32, i32)) -> Option<(f64, f64)>> {
        Ok(self.animator.new_frame(s)?)
    }
}
