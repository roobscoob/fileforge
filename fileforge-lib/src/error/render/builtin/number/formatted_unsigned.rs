use crate::error::render::{buffer::{canvas::RenderBufferCanvas, cell::tag::CellTag}, r#trait::renderable::Renderable};

use super::{separator::Separator, DIGITS_LOWER, DIGITS_UPPER};

pub struct FormattedUnsigned<'tag> {
  value: u64,
  base: usize,
  padding: usize,
  is_uppercase: bool,
  tag: Option<&'tag dyn CellTag>,
  separator: Option<Separator>
}

impl<'tag> FormattedUnsigned<'tag> {
  pub fn new(value: u64) -> FormattedUnsigned<'tag> {
    FormattedUnsigned { value, base: 10, padding: 1, is_uppercase: false, tag: None, separator: None }
  }

  pub fn with_base(mut self, base: usize) -> Self {
    self.base = base;
    self
  }

  pub fn with_padding(mut self, padding: usize) -> Self {
    self.padding = usize::max(padding, 1);
    self
  }

  pub fn with_tag(mut self, tag: &'tag dyn CellTag) -> Self {
    self.tag = Some(tag);
    self
  }

  pub fn with_uppercase(mut self) -> Self {
    self.is_uppercase = true;
    self
  }

  pub fn with_lowercase(mut self) -> Self {
    self.is_uppercase = false;
    self
  }

  pub fn with_separator(mut self, width: usize, text: &'static str) -> Self {
    self.separator = Some(Separator { width, text });
    self
  }

  pub fn length_excluding_separator(&self) -> usize {
    let mut i = 0;

    let mut value = self.value;

    while value > 0 {
      value /= self.base as u64;
      i += 1;
    }

    usize::max(i, self.padding)
  }

  pub fn length(&self) -> usize {
    let mut i = 0;
    let mut idx = 0;
    let length = self.length_excluding_separator();

    let mut value = self.value;

    while value > 0 {
      value /= self.base as u64;

      if let Some(ref separator) = self.separator {
        if i != 0 {
          let character_index = length - idx;
  
          if character_index % separator.width == 0 {
            i += 1;
          }
        }
      }

      i += 1;
      idx += 1;
    }

    usize::max(i, self.padding)
  }
}

impl<'t, 'tag> Renderable<'t> for FormattedUnsigned<'t> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    let length = self.length();

    let digits = if self.is_uppercase { DIGITS_UPPER } else { DIGITS_LOWER };

    for _ in 0..self.length() {
      if let Some(tag) = self.tag.as_ref() {
        canvas.set_tagged_char(digits[0], *tag);
      } else {
        canvas.set_char(digits[0]);
      }
    }
    
    let mut value = self.value;

    let mut index = 0;

    canvas.cursor_left_by(1);

    while value > 0 {
      if let Some(ref separator) = self.separator {
        if index != 0 {
          let character_index = length - index;
  
          if character_index % separator.width == 0 {
            if let Some(tag) = self.tag.as_ref() {
              canvas.set_tagged_str(&separator.text, *tag);
            } else {
              canvas.set_str(&separator.text);
            }
            canvas.cursor_left_by(2);
          }
        }
      }

      let digit = value % (self.base as u64);
      value /= self.base as u64;

      if let Some(tag) = self.tag.as_ref() {
        canvas.set_tagged_char(digits[digit as usize], *tag);
      } else {
        canvas.set_char(digits[digit as usize]);
      }
      canvas.cursor_left_by(2);
       
      index += 1;
    };

    canvas.set_position(canvas.start_position);
    canvas.cursor_right_by(self.length());

    Ok(())
  }
}