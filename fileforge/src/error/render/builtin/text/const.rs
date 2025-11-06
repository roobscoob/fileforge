use unicode_segmentation::UnicodeSegmentation;

use crate::error::render::{
  buffer::{canvas::RenderBufferCanvas, cell::tag::CellTag},
  r#trait::renderable::Renderable,
};

pub struct ConstText {
  content: &'static str,
  tag: &'static dyn CellTag,
  split_on_words: bool,
}

impl ConstText {
  pub const fn new(content: &'static str, tag: &'static dyn CellTag) -> ConstText {
    ConstText {
      content,
      tag,
      split_on_words: true,
    }
  }

  pub const fn without_split_on_words(mut self) -> Self {
    self.split_on_words = false;
    self
  }
}

impl Renderable<'static> for ConstText {
  fn render_into<'r, 'c>(
    &self,
    canvas: &mut RenderBufferCanvas<'r, 'c, 'static>,
  ) -> Result<(), ()> {
    let start = canvas.get_position();

    if self.split_on_words {
      for chunk in self.content.split_word_bounds() {
        if chunk == "\n" {
          canvas.cursor_down().set_column(start.column());
          continue;
        }

        if canvas.position.right(chunk.len()).column() > canvas.buffer.width() {
          canvas.cursor_down().set_column(start.column());
        }

        for grapheme in chunk.graphemes(true) {
          if !canvas.set_tagged_char(grapheme, self.tag) {
            canvas
              .cursor_down()
              .set_column(start.column())
              .set_char(grapheme);
          };
        }
      }
    } else {
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
    }

    Ok(())
  }
}
