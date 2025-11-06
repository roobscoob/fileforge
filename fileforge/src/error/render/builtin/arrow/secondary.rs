use crate::error::render::{
  buffer::{canvas::RenderBufferCanvas, cell::tag::builtin::arrow::ARROW_BODY},
  builtin::transformation::Transformation,
  r#trait::renderable::Renderable,
};

pub struct SecondaryArrow {
  pub(crate) height: usize,
  pub(crate) transformation: Option<Transformation>,
  pub(crate) replace_last: bool,
}

impl<'t> Renderable<'t> for SecondaryArrow {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.set_tagged_char(if self.replace_last { "├" } else { "│" }, &ARROW_BODY);
    canvas.cursor_left().cursor_down();

    for _ in 0..self.height {
      canvas.set_tagged_char("│", &ARROW_BODY);
      canvas.cursor_left().cursor_down();
    }

    canvas.set_tagged_char("╰", &ARROW_BODY);

    if let Some(ref transformation) = self.transformation {
      canvas.write(transformation)?;
    } else {
      canvas.set_tagged_str("->", &ARROW_BODY);
    }

    canvas.cursor_right();

    Ok(())
  }
}
