use unicode_segmentation::UnicodeSegmentation;

use crate::error::render::{
  buffer::{canvas::RenderBufferCanvas, cell::tag::CellTag},
  r#trait::renderable::Renderable,
};

pub enum TextSegment<'l, 't> {
  Renderable(&'l dyn Renderable<'t>),
  Segment(&'l str, &'t dyn CellTag),
}

pub struct Text<'l, 't> {
  segments: heapless::Vec<TextSegment<'l, 't>, 0x10>,
}

impl<'l, 't> Text<'l, 't> {
  pub fn new() -> Text<'l, 't> {
    Text {
      segments: heapless::Vec::new(),
    }
  }

  pub fn with(mut self, value: &'l dyn Renderable<'t>) -> Self {
    self
      .segments
      .push(TextSegment::Renderable(value))
      .map_err(|_| {})
      .expect("Failed to push Renderable, Text full.");
    self
  }

  pub fn push(mut self, text: &'l str, tag: &'t dyn CellTag) -> Self {
    self
      .segments
      .push(TextSegment::Segment(text, tag))
      .map_err(|_| {})
      .expect("Failed to push Segment. Text full.");
    self
  }
}

impl<'l, 't> Renderable<'t> for Text<'l, 't> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    let start = canvas.get_position();
    for element in self.segments.iter() {
      match element {
        TextSegment::Renderable(renderable) => {
          canvas.write(*renderable)?;
        }
        TextSegment::Segment(text, tag) => {
          for grapheme in text.graphemes(true) {
            if grapheme == "\n" {
              canvas.cursor_down().set_column(start.column());
              continue;
            }

            if !canvas.set_tagged_char(grapheme, *tag) {
              canvas
                .cursor_down()
                .set_column(start.column())
                .set_char(grapheme);
            };
          }
        }
      }
    }

    Ok(())
  }
}
