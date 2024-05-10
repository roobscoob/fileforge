use crate::error::render::buffer::canvas::RenderBufferCanvas;

pub trait Renderable<'t> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()>;
}