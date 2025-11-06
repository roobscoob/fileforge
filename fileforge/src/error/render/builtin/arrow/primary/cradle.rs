use crate::error::render::{
  buffer::{canvas::RenderBufferCanvas, cell::tag::builtin::arrow::CRADLE},
  r#trait::renderable::Renderable,
};

pub(super) struct Cradle {
  pub width: usize,
}

impl<'t> Renderable<'t> for Cradle {
  fn render_into<'c, 'r>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    if canvas.try_cursor_left() {
      canvas.set_tagged_char("╰", &CRADLE);
    }

    for _ in 0..self.width {
      canvas.set_tagged_char("─", &CRADLE);
    }

    canvas.set_tagged_char("╯", &CRADLE);

    Ok(())
  }
}
