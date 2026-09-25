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

  /// How many digits are shown, including padding zeros.
  fn digit_count(&self) -> usize {
    let mut count = 0;
    let mut value = self.value;

    while value > 0 {
      value /= self.base as u128;
      count += 1;
    }

    usize::max(count, self.padding)
  }

  /// How many separators are shown between the digits.
  fn separator_count(&self) -> usize {
    match self.separator {
      Some(separator) if separator.width > 0 => (self.digit_count() - 1) / separator.width,
      _ => 0,
    }
  }

  pub fn length_excluding_separator(&self) -> usize {
    self.digit_count() + self.prefix.map(|v| v.len()).unwrap_or(0)
  }

  pub fn length(&self) -> usize {
    self.length_excluding_separator() + self.separator_count() * self.separator.map(|separator| separator.text.len()).unwrap_or(0)
  }
}

impl<'t, 'tag> Renderable<'t> for FormattedUnsigned<'t> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    let write = |canvas: &mut RenderBufferCanvas<'r, 'c, 't>, text: &str| match self.tag {
      Some(tag) => canvas.set_tagged_str(text, tag),
      None => canvas.set_str(text),
    };

    if let Some(prefix) = self.prefix {
      write(canvas, prefix);
    }

    let digits = if self.is_uppercase { DIGITS_UPPER } else { DIGITS_LOWER };
    let base = self.base as u128;
    let count = self.digit_count();

    // Most significant digit first. `place` counts digits from the right, so separators group
    // digits from the right (1,234 rather than 123,4).
    for place in (0..count).rev() {
      let digit = match base.checked_pow(place as u32) {
        Some(power) => (self.value / power) % base,
        // A power this large exceeds the value, so this is a padding zero.
        None => 0,
      };

      write(canvas, digits[digit as usize]);

      if let Some(separator) = self.separator {
        if place != 0 && separator.width != 0 && place % separator.width == 0 {
          write(canvas, separator.text);
        }
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::{string::String, vec, vec::Vec};

  use super::FormattedUnsigned;
  use crate::error::render::{
    buffer::{
      cell::{tag::context::RenderMode, RenderBufferCell},
      RenderBuffer,
    },
    position::RenderPosition,
  };

  fn render(number: FormattedUnsigned<'static>) -> String {
    let mut cells = vec![RenderBufferCell::default(); 80];
    let mut buffer = RenderBuffer::new(&mut cells, 80, 0);
    let summary = buffer.canvas_at(RenderPosition::zero()).write(&number).unwrap();
    assert_eq!(summary.end_position.column(), number.length(), "length() must match what is rendered");

    let mut out = String::new();
    buffer.flush_into(&mut out, RenderMode::PlainText).unwrap();
    out.lines().next().unwrap().trim_end().into()
  }

  #[test]
  fn separators_group_digits_from_the_right() {
    let cases: Vec<(u128, &str)> = vec![
      (0, "0"),
      (7, "7"),
      (999, "999"),
      (1_234, "1,234"),
      (12_345, "12,345"),
      (123_456, "123,456"),
      (1_004_582, "1,004,582"),
      (u64::MAX as u128, "18,446,744,073,709,551,615"),
    ];

    for (value, expected) in cases {
      assert_eq!(render(FormattedUnsigned::new(value).separator(3, ",")), expected);
    }
  }

  #[test]
  fn prefix_padding_and_base() {
    assert_eq!(render(FormattedUnsigned::new(0xA).base(16).uppercase().padding(2).prefix("0x")), "0x0A");
    assert_eq!(render(FormattedUnsigned::new(0xBEEF).base(16).prefix("0x")), "0xbeef");
    assert_eq!(render(FormattedUnsigned::new(5).padding(4)), "0005");
    assert_eq!(render(FormattedUnsigned::new(1234).padding(6).separator(3, ",")), "001,234");
  }
}
