use core::num::NonZero;

use fileforge_macros::story;
use fileforge::{diagnostic::pool::DiagnosticPoolProvider, error::{report::Report, FileforgeError}};
use fileforge::error::render::{buffer::cell::tag::builtin::report::REPORT_ERROR_TEXT, builtin::text::r#const::ConstText};
use crate::report::{CORRUPTED};

pub struct FilenameAttributes {
  pub sequence: NonZero<u8>,
  pub hash_index: u32,
}

#[story("zero sequence", FilenameAttributesError::ZeroSequence)]
pub enum FilenameAttributesError {
  ZeroSequence,
}

const ZERO_SEQUENCE: ConstText = ConstText::new("The entry's filename attributes are set, but their first byte (a collision index, which starts at 1) is 0.", &REPORT_ERROR_TEXT);

impl FileforgeError for FilenameAttributesError {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::ZeroSequence => Report::new::<Self>(provider, &"Invalid SFAT filename attributes")
        .with_info_line(&ZERO_SEQUENCE)
        .with_flag_line(&CORRUPTED)
        .apply(callback),
    }
  }
}

impl FilenameAttributes {
  pub fn from_bits(value: u32) -> Result<Option<FilenameAttributes>, FilenameAttributesError> {
    if value == 0 {
      Ok(None)
    } else {
      let sequence = NonZero::new((value >> 24) as u8).ok_or(FilenameAttributesError::ZeroSequence)?;
      let hash_index = value & 0xFFFFFF;

      Ok(Some(FilenameAttributes { sequence, hash_index }))
    }
  }
}
