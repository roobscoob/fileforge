use core::fmt::Write;

use self::{
  canvas::RenderBufferCanvas,
  cell::{
    tag::context::{CellTagContext, RenderMode},
    RenderBufferCell,
  },
};

use super::{grapheme::Grapheme, position::RenderPosition};

pub mod canvas;
pub mod cell;

pub struct RenderBuffer<'buffer, 'tag> {
  buffer: &'buffer mut [RenderBufferCell<'tag>],
  line_width: usize,
  line_offset: usize,
  height: usize,
  lowest_written_line: usize,
  pub(crate) highest_written_column: usize,
  is_dry: bool,
}

impl<'buffer, 'tag> RenderBuffer<'buffer, 'tag> {
  pub fn with_stack_alloc<const SIZE: usize, T>(
    line_width: usize,
    line_offset: usize,
    callback: impl FnOnce(RenderBuffer) -> T,
  ) -> T {
    let mut raw_buffer: [RenderBufferCell<'tag>; SIZE] = [Default::default(); SIZE];

    callback(RenderBuffer::new(&mut raw_buffer, line_width, line_offset))
  }

  pub fn new(
    buffer: &'buffer mut [RenderBufferCell<'tag>],
    line_width: usize,
    line_offset: usize,
  ) -> Self {
    let buffer_len = buffer.len();

    Self {
      buffer,
      line_width,
      line_offset,
      height: {
        if buffer_len < line_width {
          panic!(
            "Buffer length {} is too small for line width {}",
            buffer_len, line_width
          );
        }

        buffer_len / line_width
      },
      lowest_written_line: 0,
      highest_written_column: 0,
      is_dry: false,
    }
  }

  pub fn dry() -> Self {
    Self {
      buffer: &mut [],
      line_width: usize::max_value(),
      line_offset: 0,
      height: 0,
      lowest_written_line: 0,
      highest_written_column: 0,
      is_dry: true,
    }
  }

  pub fn can_set_char(&self, position: RenderPosition, c: Grapheme) -> bool {
    if (position.column() + c.width()) > self.line_width {
      return false;
    }

    true
  }

  /// returns `true` if the write was successful
  /// returns `false` if the write failed
  pub fn set_char(&mut self, position: RenderPosition, c: RenderBufferCell<'tag>) -> bool {
    if position.line() > self.lowest_written_line {
      self.lowest_written_line = position.line();
    }

    if position.column() > self.highest_written_column {
      self.highest_written_column = position.column();
    }

    if (position.column() + c.contents().width()) > self.line_width {
      return false;
    }

    if self.is_dry {
      return true;
    }

    // render buffer rendering relies on incremental rendering thus
    // characters written to the buffer outside of the current
    // scope are ignored, and will be rendered in another iteration
    if position.line() >= (self.height + self.line_offset) {
      return true;
    }
    if position.line() < self.line_offset {
      return true;
    }

    let index = (position.line() - self.line_offset) * self.line_width + position.column();

    self.buffer[index] = c;

    return true;
  }

  pub fn width(&self) -> usize { self.line_width }

  pub fn flush_into(
    &mut self,
    into: &mut dyn Write,
    mode: RenderMode,
  ) -> Result<(), core::fmt::Error> {
    if self.is_dry {
      return Ok(());
    }

    for line in self.buffer.chunks_exact(self.line_width) {
      let mut previous_typename: Option<&'static str> = None;

      let mut skip_count: usize = 0;

      for (cell, index) in line.iter().zip(0..) {
        if skip_count > 0 {
          skip_count -= 1;
          continue;
        }

        let my_cell_tag = cell.tag();

        if let Some(cell_tag) = my_cell_tag {
          cell_tag.render_into(
            into,
            *cell.contents(),
            CellTagContext {
              mode,
              previous_has_same_typename: previous_typename
                .map(|v| v == cell_tag.get_name())
                .unwrap_or(false),
              next_has_same_typename: line
                .get(index + 1)
                .map(|v| v.tag().map(|t| t.get_name() == cell_tag.get_name()))
                .flatten()
                .unwrap_or(false),
            },
          )?
        } else {
          let contents = cell.contents();

          if contents.is_empty() {
            into.write_str(" ")?;
          } else {
            into.write_str(contents.as_str())?;
          }
        }

        if cell.contents().width() == 0 {
          skip_count = 0
        } else {
          skip_count = cell.contents().width() - 1;
        }

        previous_typename = my_cell_tag.map(|v| v.get_name())
      }

      into.write_str("\n")?;
    }

    Ok(())
  }

  pub fn canvas_at<'a>(
    &'a mut self,
    position: RenderPosition,
  ) -> RenderBufferCanvas<'a, 'buffer, 'tag> {
    RenderBufferCanvas {
      buffer: self,
      position,
      start_position: position,
    }
  }

  pub fn is_empty(&self) -> bool { self.line_offset > self.lowest_written_line }
}
