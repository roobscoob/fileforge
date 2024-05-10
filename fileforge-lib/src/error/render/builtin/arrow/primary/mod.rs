use crate::error::render::{buffer::{canvas::RenderBufferCanvas, cell::tag::builtin::arrow::ARROW_BODY}, builtin::transformation::Transformation, r#trait::renderable::Renderable};

use self::cradle::Cradle;

pub mod cradle;

pub struct PrimaryArrow {
  pub (crate) cradle_width: usize,
  pub (crate) indent: usize,
  pub (crate) transformation: Option<Transformation>,
}

impl<'t> Renderable<'t> for PrimaryArrow {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.write(&Cradle { width: self.cradle_width })?;

    if canvas.get_start_position().column() + self.cradle_width <= self.indent {
      panic!("Down and Right (DaR) arrows are unsupported");
    }

    canvas.set_position(canvas.get_start_position());

    if canvas.get_start_position().column() <= self.indent {
      // Down Exclusive mode.

      // Safety: will never underflow since self.indent >= start_position.column
      let cradle_turn_index = self.indent - canvas.get_start_position().column();

      canvas.cursor_right_by(cradle_turn_index);
      canvas.set_tagged_char("┬", &ARROW_BODY);
      canvas.cursor_left().cursor_down();
      canvas.set_tagged_char("╰", &ARROW_BODY);

      if let Some(ref transformation) = self.transformation {
        canvas.write(transformation)?;
      } else {
        canvas.set_tagged_str("->", &ARROW_BODY);
      }

      canvas.cursor_right();
      
      return Ok(());
    };

    // DaL (down and left) mode
    // since this means the indent is always less than the canvas start position column
    // we can always place 1 padding space and move left.
    // the tricky part is figuring out how far left we need to move by.

    canvas.cursor_right();

    let end_straight_column = canvas.get_position().column();

    canvas.set_tagged_char("┬", &ARROW_BODY);

    // the canvas writes LtR so we should too... probably
    canvas.set_column(self.indent).cursor_down();

    canvas.set_tagged_char("╭", &ARROW_BODY);

    for _ in canvas.get_position().column()..end_straight_column {
      canvas.set_tagged_char("─", &ARROW_BODY);
    }

    canvas.set_tagged_char("╯", &ARROW_BODY);
    
    canvas.cursor_down().set_column(self.indent);

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