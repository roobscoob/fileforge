use crate::error::render::{buffer::{canvas::RenderBufferCanvas, cell::tag::builtin::transformation::{TRANSFORMATION_NAME, TRANSFORMATION_SEPARATOR}}, r#trait::renderable::Renderable};

#[derive(Clone, Copy, Debug)]
pub struct Transformation {
  pub (crate) name: &'static str,
}

impl<'t> Renderable<'t> for Transformation {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.set_tagged_str(">-[", &TRANSFORMATION_SEPARATOR);
    canvas.set_tagged_str(&self.name, &TRANSFORMATION_NAME);
    canvas.set_tagged_str("]->", &TRANSFORMATION_SEPARATOR);

    Ok(())
  }
}