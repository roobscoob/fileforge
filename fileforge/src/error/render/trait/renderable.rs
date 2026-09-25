use crate::error::render::buffer::canvas::RenderBufferCanvas;

pub trait Renderable<'tag> {
  fn render_into<'buffer_reference, 'buffer_contents>(&self, canvas: &mut RenderBufferCanvas<'buffer_reference, 'buffer_contents, 'tag>) -> Result<(), ()>;
}

impl<'a, 't, O: Renderable<'t>> Renderable<'t> for &'a O {
  fn render_into<'buffer_reference, 'buffer_contents>(&self, canvas: &mut RenderBufferCanvas<'buffer_reference, 'buffer_contents, 't>) -> Result<(), ()> {
    (**self).render_into(canvas)
  }
}
