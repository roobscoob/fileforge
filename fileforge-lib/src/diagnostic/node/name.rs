use core::fmt::Debug;

use unicode_segmentation::UnicodeSegmentation;

use crate::error::render::builtin::number::formatted_unsigned::FormattedUnsigned;

#[derive(Clone, Copy)]
pub struct DiagnosticNodeName<const SIZE: usize> {
  // UNSAFE: We need to validate this is *ALWAYS* valid utf-8
  // We can't use Heapless because it doesn't implement Copy
  // CONTENTS should also be initialized to null
  contents: [u8; SIZE],
  used_length: usize,
  // NOTE: total_length can be LONGER than SIZE!!!
  // This occurs if you attempt to write a string longer
  // than we can store.
  // when rendering, show an ellipsis: e.g. "Hello" -> DiagnosticNodeName<3> -> "Hel..."
  total_length: usize,
}

const CHARS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];

impl<const SIZE: usize> DiagnosticNodeName<SIZE> {
  pub fn from(text: &str) -> DiagnosticNodeName<SIZE> {
    let mut name = DiagnosticNodeName {
      contents: [0; SIZE],
      used_length: 0,
      total_length: text.as_bytes().len(),
    };

    for grapheme in text.graphemes(true) {
      let bytes = grapheme.as_bytes();

      if bytes.len() > name.remaining_size() {
        break;
      }

      name.contents[name.used_length..(name.used_length + bytes.len())].copy_from_slice(bytes);

      name.used_length += bytes.len();
    }

    name
  }

  pub fn from_index(index: u64) -> DiagnosticNodeName<SIZE> {
    let mut name = DiagnosticNodeName {
      contents: [0; SIZE],
      used_length: 0,
      total_length: 0,
    };

    name.push(b'[');

    let mut my_index = index;

    if SIZE > 0 {
      if my_index == 0 {
        name.push(b'0');
      }

      while my_index > 0 {
        let current_digit_index = my_index / (10u64.pow(my_index.ilog10()));
        let current_digit = CHARS.get(current_digit_index as usize).unwrap();
        my_index = my_index % (10u64.pow(my_index.ilog10()));
        name.push(*current_digit);
      }
    };

    name.push(b']');

    name
  }

  pub fn from_named_index(name: &'static str, index: u64) -> DiagnosticNodeName<SIZE> {
    let mut name = DiagnosticNodeName::from(name);

    name.push(b'[');

    let mut my_index = index;

    if SIZE > 0 {
      if my_index == 0 {
        name.push(b'0');
      }

      while my_index > 0 {
        let current_digit_index = my_index / (10u64.pow(my_index.ilog10()));
        let current_digit = CHARS.get(current_digit_index as usize).unwrap();
        my_index = my_index % (10u64.pow(my_index.ilog10()));
        name.push(*current_digit);
      }
    };

    name.push(b']');

    name
  }

  fn push(&mut self, char: u8) {
    self.total_length += 1;

    if self.used_length < SIZE {
      self.contents[self.used_length] = char;
      self.used_length += 1;
    }
  }

  fn remaining_size(&self) -> usize { SIZE - self.used_length }

  pub fn show_ellipsis(&self) -> bool { self.total_length > self.used_length }

  pub fn as_str(&self) -> &str { unsafe { core::str::from_utf8_unchecked(&self.contents[0..self.used_length]) } }
}

impl<const SIZE: usize> Eq for DiagnosticNodeName<SIZE> {}
impl<const SIZE: usize> PartialEq for DiagnosticNodeName<SIZE> {
  fn eq(&self, other: &Self) -> bool { self.as_str() == other.as_str() }
}

impl<const SIZE: usize> Debug for DiagnosticNodeName<SIZE> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.as_str())?;

    if self.show_ellipsis() {
      f.write_str("...")?;
    }

    Ok(())
  }
}

impl<const SIZE: usize> From<&str> for DiagnosticNodeName<SIZE> {
  fn from(value: &str) -> Self { DiagnosticNodeName::from(value) }
}
