use unicode_segmentation::UnicodeSegmentation;

use crate::error::render::{grapheme::Grapheme, position::RenderPosition, r#trait::renderable::Renderable};

use self::summary::RenderBufferCanvasSummary;

use super::{cell::{tag::CellTag, RenderBufferCell}, RenderBuffer};

pub mod summary;

pub struct RenderBufferCanvas<'buffer_reference, 'buffer_contents, 'tag> {
  pub(crate) buffer: &'buffer_reference mut RenderBuffer<'buffer_contents, 'tag>,
  pub(crate) start_position: RenderPosition,
  pub(crate) position: RenderPosition,
}

impl<'reference, 'contents, 'tag> RenderBufferCanvas<'reference, 'contents, 'tag> {
  pub fn write(&mut self, renderable: &dyn Renderable<'tag>) -> Result<RenderBufferCanvasSummary, ()> {
    let mut new_canvas = RenderBufferCanvas { buffer: self.buffer, position: self.position, start_position: self.position };

    renderable.render_into(&mut new_canvas)?;

    let result = new_canvas.end();

    self.position = result.end_position;

    Ok(result)
  }

  pub fn set_char(&mut self, str: &str) -> bool {
    let cell = RenderBufferCell::from_str(str);
    let result = self.buffer.set_char(self.position, cell);
    self.position = self.position.right(cell.width());
    result
  }

  pub fn set_str(&mut self, str: &str) {
    let start = self.get_position();

    for grapheme in str.graphemes(true) {
      if !self.set_char(grapheme) {
        self.cursor_down().set_column(start.column()).set_char(grapheme);
      };
    };
  }

  pub fn set_tagged_char(&mut self, str: &str, tag: &'tag dyn CellTag) -> bool {
    let cell = RenderBufferCell::from_str(str).with_tag(tag);
    let result = self.buffer.set_char(self.position, cell);
    self.position = self.position.right(cell.width());
    result
  }

  pub fn set_tagged_str(&mut self, str: &str, tag: &'tag dyn CellTag) {
    let start = self.get_position();

    for grapheme in str.graphemes(true) {
      if !self.set_tagged_char(grapheme, tag) {
        self.cursor_down().set_column(start.column()).set_tagged_char(grapheme, tag);
      };
    };
  }

  pub fn set_grapheme(&mut self, grapheme: Grapheme) -> bool {
    let cell = RenderBufferCell::new(grapheme);
    let result = self.buffer.set_char(self.position, cell);
    self.position = self.position.right(cell.width());
    result
  }

  pub fn set_tagged_grapheme(&mut self, grapheme: Grapheme, tag: &'tag dyn CellTag) -> bool {
    let cell = RenderBufferCell::new(grapheme).with_tag(tag);
    let result = self.buffer.set_char(self.position, cell);
    self.position = self.position.right(cell.width());
    result
  }

  fn end(self) -> RenderBufferCanvasSummary {
    RenderBufferCanvasSummary {
      end_position: self.position,
      start_position: self.start_position,
    }
  }

  pub fn get_start_position(&self) -> RenderPosition { self.start_position }
  pub fn get_position(&self) -> RenderPosition {self.position}
  pub fn set_position(&mut self, position: RenderPosition) -> &mut Self { self.position = position; self }
  pub fn set_column(&mut self, column: usize) -> &mut Self { self.position = RenderPosition::new(self.position.line(), column); self }
  pub fn set_line(&mut self, line: usize) -> &mut Self { self.position = RenderPosition::new(line, self.position.column()); self }
  
  pub fn cursor_down(&mut self) -> &mut Self { self.position = self.position.down(1); self }
  pub fn cursor_down_by(&mut self, count: usize) -> &mut Self { self.position = self.position.down(count); self }
  pub fn try_cursor_up(&mut self) -> bool { if let Some(new_position) = self.position.try_up(1) { self.position = new_position; true } else { false }}
  pub fn try_cursor_up_by(&mut self, count: usize) -> bool { if let Some(new_position) = self.position.try_up(count) { self.position = new_position; true } else { false }}
  pub fn cursor_up(&mut self) -> &mut Self { self.position = self.position.try_up(1).expect("Expected to be able to move the cursor up"); self }
  pub fn cursor_up_by(&mut self, count: usize) -> &mut Self { self.position = self.position.try_up(count).expect("Expected to be able to move the cursor up"); self }
  pub fn try_cursor_left(&mut self) -> bool { if let Some(new_position) = self.position.try_left(1) { self.position = new_position; true } else { false }}
  pub fn try_cursor_left_by(&mut self, count: usize) -> bool { if let Some(new_position) = self.position.try_left(count) { self.position = new_position; true } else { false }}
  pub fn cursor_left(&mut self) -> &mut Self { self.position = self.position.try_left(1).expect("Expected to be able to move the cursor left"); self }
  pub fn cursor_left_by(&mut self, count: usize) -> &mut Self { self.position = self.position.try_left(count).expect("Expected to be able to move the cursor left"); self }
  pub fn cursor_right(&mut self) -> &mut Self { self.position = self.position.right(1); self }
  pub fn cursor_right_by(&mut self, count: usize) -> &mut Self { self.position = self.position.right(count); self }
}