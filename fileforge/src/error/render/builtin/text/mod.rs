pub mod r#const;

use unicode_segmentation::UnicodeSegmentation;

use crate::error::render::{
  buffer::{canvas::RenderBufferCanvas, cell::tag::CellTag},
  r#trait::renderable::Renderable,
};

pub enum TextSegment<'l, 't> {
  Renderable(&'l dyn Renderable<'t>),
  Segment(&'l str, Option<&'t dyn CellTag>),
}

pub struct Text<'l, 't> {
  segments: heapless::Vec<TextSegment<'l, 't>, 0x10>,
  split_on_words: bool,
}

impl<'l, 't> Text<'l, 't> {
  pub const fn new() -> Self {
    Text {
      segments: heapless::Vec::new(),
      split_on_words: true,
    }
  }

  pub fn of_tagged(text: &'l str, tag: &'t dyn CellTag) -> Self {
    let t = Self::new();

    t.push_tagged(text, tag)
  }

  pub fn of(text: &'l str) -> Self {
    let t = Self::new();

    t.push(text)
  }

  pub const fn without_split_on_words(mut self) -> Self {
    self.split_on_words = false;
    self
  }

  pub fn with(mut self, value: &'l dyn Renderable<'t>) -> Self {
    self.segments.push(TextSegment::Renderable(value)).map_err(|_| {}).expect("Failed to push Renderable, Text full.");
    self
  }

  pub fn push_tagged(mut self, text: &'l str, tag: &'t dyn CellTag) -> Self {
    self.segments.push(TextSegment::Segment(text, Some(tag))).map_err(|_| {}).expect("Failed to push Segment. Text full.");
    self
  }

  pub fn push(mut self, text: &'l str) -> Self {
    self.segments.push(TextSegment::Segment(text, None)).map_err(|_| {}).expect("Failed to push Segment. Text full.");
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
          if self.split_on_words {
            for chunk in text.split_word_bounds() {
              if chunk == "\n" {
                canvas.cursor_down().set_column(start.column());
                continue;
              }

              if canvas.position.right(chunk.len()).column() > canvas.buffer.width() {
                canvas.cursor_down().set_column(start.column());
              }

              for grapheme in chunk.graphemes(true) {
                if let Some(tag) = tag {
                  if !canvas.set_tagged_char(grapheme, *tag) {
                    canvas.cursor_down().set_column(start.column()).set_tagged_char(grapheme, *tag);
                  };
                } else {
                  if !canvas.set_char(grapheme) {
                    canvas.cursor_down().set_column(start.column()).set_char(grapheme);
                  };
                }
              }
            }
          } else {
            for grapheme in text.graphemes(true) {
              if grapheme == "\n" {
                canvas.cursor_down().set_column(start.column());
                continue;
              }

              if let Some(tag) = tag {
                if !canvas.set_tagged_char(grapheme, *tag) {
                  canvas.cursor_down().set_column(start.column()).set_tagged_char(grapheme, *tag);
                };
              } else {
                if !canvas.set_char(grapheme) {
                  canvas.cursor_down().set_column(start.column()).set_char(grapheme);
                };
              }
            }
          }
        }
      }
    }

    Ok(())
  }
}
