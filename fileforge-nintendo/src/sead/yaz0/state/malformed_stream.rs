use fileforge::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    render::{buffer::cell::tag::builtin::report::REPORT_ERROR_TEXT, builtin::number::formatted_unsigned::FormattedUnsigned},
    report::Report,
    FileforgeError,
  },
};
use fileforge_macros::{story, text};

use crate::report::CORRUPTED;

#[story("back-reference before the start of the output", MalformedStream::SeekbackOutOfBounds { seekback_offset: 12, seekback_size: 4 })]
#[derive(Debug)]
pub enum MalformedStream {
  SeekbackOutOfBounds { seekback_offset: u16, seekback_size: u16 },
}

impl FileforgeError for MalformedStream {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::SeekbackOutOfBounds { seekback_offset, seekback_size } => {
        let offset = FormattedUnsigned::new(*seekback_offset as u128).separator(3, ",");
        let available = FormattedUnsigned::new(*seekback_size as u128).separator(3, ",");

        let reach_text = text!([&REPORT_ERROR_TEXT] "A back-reference reaches {&offset} bytes back into the decompressed data.");
        let available_text = text!(
          { *seekback_size == 1 }
            [&REPORT_ERROR_TEXT] "Only 1 byte is available to refer back to.",

          [&REPORT_ERROR_TEXT] "Only {&available} bytes are available to refer back to."
        );

        Report::new::<Self>(provider, &"Corrupt Yaz0 data")
          .with_info_line(&reach_text)
          .with_info_line(&available_text)
          .with_flag_line(&CORRUPTED)
          .apply(callback)
      }
    }
  }
}
