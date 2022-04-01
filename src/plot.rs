use crate::DrawResult;
const MAX_LEN: usize = 20;
impl super::Worker {
    pub(crate) fn draw(
        &mut self,
        s: Vec<(f64, f64)>,
    ) -> DrawResult<impl Fn((i32, i32)) -> Option<(f64, f64)>> {
        self.history.push(s.iter().map(|x| x.1).collect());
        let _s = self.history.draw()?;
        Ok(self.animator.new_frame(s)?)
    }
}
