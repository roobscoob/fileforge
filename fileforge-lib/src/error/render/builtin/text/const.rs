use unicode_segmentation::UnicodeSegmentation;

use crate::error::render::{
  buffer::{canvas::RenderBufferCanvas, cell::tag::CellTag},
  r#trait::renderable::Renderable,
};

pub struct ConstText {
  content: &'static str,
  tag: &'static dyn CellTag,
}

impl ConstText {
  pub const fn new(content: &'static str, tag: &'static dyn CellTag) -> ConstText {
    ConstText { content, tag }
  }
}

impl Renderable<'static> for ConstText {
  fn render_into<'r, 'c>(
    &self,
    canvas: &mut RenderBufferCanvas<'r, 'c, 'static>,
  ) -> Result<(), ()> {
    let start = canvas.get_position();

    for grapheme in self.content.graphemes(true) {
      if grapheme == "\n" {
        canvas.cursor_down().set_column(start.column());
        continue;
      }

      if !canvas.set_tagged_char(grapheme, self.tag) {
        canvas
          .cursor_down()
          .set_column(start.column())
          .set_char(grapheme);
      };
    }
    Ok(())
  }
}
