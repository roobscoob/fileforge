use crate::error::render::buffer::canvas::RenderBufferCanvas;

pub trait Renderable<'tag> {
  fn render_into<'buffer_reference, 'buffer_contents>(&self, canvas: &mut RenderBufferCanvas<'buffer_reference, 'buffer_contents, 'tag>) -> Result<(), ()>;
}
