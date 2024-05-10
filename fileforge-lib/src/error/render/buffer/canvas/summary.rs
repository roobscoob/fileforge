use crate::error::render::position::RenderPosition;

#[derive(Clone, Copy)]
pub struct RenderBufferCanvasSummary {
  pub start_position: RenderPosition,
  pub end_position: RenderPosition,
}

impl RenderBufferCanvasSummary {
  pub fn get_line_height(&self) -> usize {
    1 + (self.end_position.line() - self.start_position.line())
  }
}