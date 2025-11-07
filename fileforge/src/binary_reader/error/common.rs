use crate::error::render::{
  buffer::cell::tag::{builtin::report::REPORT_FLAG_LINE_TEXT, CellTag},
  builtin::{number::formatted_unsigned::FormattedUnsigned, text::r#const::ConstText},
};

pub const LOW_LEVEL_ERROR: &'static ConstText = const_text!([&REPORT_FLAG_LINE_TEXT] "This is a low-level error, intended to be consumed by higher-level error handling code. This error is not intended to be displayed to the user. If you're seeing this error and *not* a library author, it may be confusing. Please report this error to the library author.");

pub enum SeekOffset {
  Underflow { base_offset: u64, subtract: u64 },
  Overflowed { base_offset: u64, add: u64 },
  InBounds(u64),
}

impl SeekOffset {
  pub fn format(&self) -> FormattedUnsigned<'static> {
    match self {
      Self::InBounds(v) => FormattedUnsigned::from(v),
      Self::Overflowed { base_offset, add } => FormattedUnsigned::new((*base_offset as u128) + (*add as u128)),
      Self::Underflow { base_offset, subtract } => {
        let value = (*base_offset as i128) - (*subtract as i128);

        if value < 0 {
          FormattedUnsigned::new(value.abs() as u128).prefix("-")
        } else {
          FormattedUnsigned::new(value as u128)
        }
      }
    }
  }

  pub fn did_overflow(&self) -> bool {
    match self {
      Self::InBounds(..) => false,
      Self::Underflow { .. } => false,
      Self::Overflowed { .. } => true,
    }
  }

  pub fn did_underflow(&self) -> bool {
    match self {
      Self::InBounds(..) => false,
      Self::Underflow { .. } => true,
      Self::Overflowed { .. } => false,
    }
  }
}

pub trait ExhaustedType {
  const VALUE: Self;

  fn message(&self, tag: &'static dyn CellTag) -> ConstText;
}

pub struct Read;
pub struct Write;

impl ExhaustedType for Read {
  const VALUE: Self = Read;

  fn message(&self, tag: &'static dyn CellTag) -> ConstText {
    ConstText::new("read", tag)
  }
}

impl ExhaustedType for Write {
  const VALUE: Self = Write;

  fn message(&self, tag: &'static dyn CellTag) -> ConstText {
    ConstText::new("write", tag)
  }
}
