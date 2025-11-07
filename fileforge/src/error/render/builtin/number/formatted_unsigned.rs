use crate::{
  diagnostic::value::DiagnosticValue,
  error::render::{
    buffer::{canvas::RenderBufferCanvas, cell::tag::CellTag},
    r#trait::renderable::Renderable,
  },
};

use super::{separator::Separator, DIGITS_LOWER, DIGITS_UPPER};

#[derive(Clone, Copy)]
pub struct FormattedUnsigned<'tag> {
  value: u128,
  base: usize,
  padding: usize,
  is_uppercase: bool,
  tag: Option<&'tag dyn CellTag>,
  separator: Option<Separator>,
  prefix: Option<&'static str>,
}

impl<'tag> From<u8> for FormattedUnsigned<'tag> {
  fn from(value: u8) -> Self {
    Self::new(value as u128)
  }
}

impl<'tag> From<u16> for FormattedUnsigned<'tag> {
  fn from(value: u16) -> Self {
    Self::new(value as u128)
  }
}

impl<'tag> From<u32> for FormattedUnsigned<'tag> {
  fn from(value: u32) -> Self {
    Self::new(value as u128)
  }
}

impl<'tag> From<u64> for FormattedUnsigned<'tag> {
  fn from(value: u64) -> Self {
    Self::new(value as u128)
  }
}

impl<'tag> From<usize> for FormattedUnsigned<'tag> {
  fn from(value: usize) -> Self {
    Self::new(value as u128)
  }
}

impl<'tag, T: Into<FormattedUnsigned<'tag>> + Copy> From<&T> for FormattedUnsigned<'tag> {
  fn from(value: &T) -> Self {
    (*value).into()
  }
}

impl<'tag, 'pool, T: Into<FormattedUnsigned<'tag>> + Copy> From<DiagnosticValue<'pool, T>> for FormattedUnsigned<'tag> {
  fn from(value: DiagnosticValue<'pool, T>) -> Self {
    (*value).into()
  }
}

pub trait FormattedExt<'tag> {
  fn format(self) -> FormattedUnsigned<'tag>;
}

impl<'tag, T: Into<FormattedUnsigned<'tag>>> FormattedExt<'tag> for T {
  fn format(self) -> FormattedUnsigned<'tag> {
    self.into()
  }
}

impl<'tag> FormattedUnsigned<'tag> {
  pub fn new(value: u128) -> FormattedUnsigned<'tag> {
    FormattedUnsigned {
      value,
      base: 10,
      padding: 1,
      is_uppercase: false,
      tag: None,
      separator: None,
      prefix: None,
    }
  }

  pub fn base(mut self, base: usize) -> Self {
    self.base = base;
    self
  }

  pub fn padding(mut self, padding: usize) -> Self {
    self.padding = usize::max(padding, 1);
    self
  }

  pub fn tag(mut self, tag: &'tag dyn CellTag) -> Self {
    self.tag = Some(tag);
    self
  }

  pub fn uppercase(mut self) -> Self {
    self.is_uppercase = true;
    self
  }

  pub fn lowercase(mut self) -> Self {
    self.is_uppercase = false;
    self
  }

  pub fn separator(mut self, width: usize, text: &'static str) -> Self {
    self.separator = Some(Separator { width, text });
    self
  }

  pub fn prefix(mut self, prefix: &'static str) -> Self {
    self.prefix = Some(prefix);
    self
  }

  pub fn length_excluding_separator(&self) -> usize {
    let mut i = 0;

    let mut value = self.value;

    while value > 0 {
      value /= self.base as u128;
      i += 1;
    }

    usize::max(i, self.padding) + self.prefix.map(|v| v.len()).unwrap_or(0)
  }

  pub fn length(&self) -> usize {
    let mut i = 0;
    let mut idx = 0;
    let length = self.length_excluding_separator();

    let mut value = self.value;

    while value > 0 {
      value /= self.base as u128;

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

    usize::max(i, self.padding) + self.prefix.map(|v| v.len()).unwrap_or(0)
  }
}

impl<'t, 'tag> Renderable<'t> for FormattedUnsigned<'t> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    let mut length = self.length();

    if let Some(prefix) = self.prefix {
      if let Some(tag) = self.tag.as_ref() {
        canvas.set_tagged_str(prefix, *tag);
      } else {
        canvas.set_str(prefix);
      }

      length -= prefix.len();
    }

    let digits = if self.is_uppercase { DIGITS_UPPER } else { DIGITS_LOWER };

    for _ in 0..length {
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

      let digit = value % (self.base as u128);
      value /= self.base as u128;

      if let Some(tag) = self.tag.as_ref() {
        canvas.set_tagged_char(digits[digit as usize], *tag);
      } else {
        canvas.set_char(digits[digit as usize]);
      }
      canvas.cursor_left_by(2);

      index += 1;
    }

    canvas.set_position(canvas.start_position);
    canvas.cursor_right_by(self.length());

    Ok(())
  }
}
