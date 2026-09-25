use core::num::NonZero;

use fileforge_macros::{story, text};
use fileforge::{diagnostic::pool::DiagnosticPoolProvider, error::{report::Report, FileforgeError}};
use fileforge::error::render::{
  buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT},
  builtin::{number::formatted_unsigned::FormattedUnsigned, text::r#const::ConstText},
};
use crate::report::{CORRUPTED};

pub struct FilenameAttributes {
  pub sequence: NonZero<u8>,
  pub hash_index: u32,
}

#[story("zero sequence", FilenameAttributesError::ZeroSequence { attributes: 0x0000_1234 })]
pub enum FilenameAttributesError {
  /// `attributes` is non-zero, but its top byte (the collision index) is 0.
  ZeroSequence { attributes: u32 },
}

const ZERO_SEQUENCE: ConstText = ConstText::new("They're set, but their top byte (a collision index, which starts at 1) is 0.", &REPORT_INFO_LINE_TEXT);

impl FileforgeError for FilenameAttributesError {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::ZeroSequence { attributes } => {
        let attributes = FormattedUnsigned::new(*attributes as u128).base(16).uppercase().padding(8).prefix("0x");
        let attributes_text = text!([&REPORT_ERROR_TEXT] "The entry's filename attributes are {&attributes}.");

        Report::new::<Self>(provider, &"Invalid SFAT filename attributes")
          .with_info_line(&attributes_text)
          .with_info_line(&ZERO_SEQUENCE)
          .with_flag_line(&CORRUPTED)
          .apply(callback)
      }
    }
  }
}

impl FilenameAttributes {
  pub fn from_bits(value: u32) -> Result<Option<FilenameAttributes>, FilenameAttributesError> {
    if value == 0 {
      Ok(None)
    } else {
      let sequence = NonZero::new((value >> 24) as u8).ok_or(FilenameAttributesError::ZeroSequence { attributes: value })?;
      let hash_index = value & 0xFFFFFF;

      Ok(Some(FilenameAttributes { sequence, hash_index }))
    }
  }
}
