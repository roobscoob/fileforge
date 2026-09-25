use fileforge_macros::{story, text};
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use crate::binary_reader::error::common::LOW_LEVEL_ERROR;
use crate::error::render::buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT};
use crate::error::render::builtin::number::formatted_unsigned::FormattedUnsigned;
#[story("past the end", StreamSeekOutOfBoundsError { stream_length: 16, seek_point: 40 })]
#[derive(Debug)]
pub struct StreamSeekOutOfBoundsError {
  pub stream_length: u64,
  pub seek_point: u64,
}

impl StreamSeekOutOfBoundsError {
  pub fn assert(stream_length: u64, seek_point: u64) -> Result<(), Self> {
    if seek_point > stream_length { Err(Self { seek_point, stream_length }) } else { Ok(()) }
  }
}

impl FileforgeError for StreamSeekOutOfBoundsError {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    let seek_point = FormattedUnsigned::new(self.seek_point as u128).base(16).uppercase().prefix("0x");
    let stream_length = FormattedUnsigned::new(self.stream_length as u128).separator(3, ",");

    let seek_text = text!([&REPORT_ERROR_TEXT] "Tried to move to {&seek_point}, past the end of the stream.");
    let length_text = text!([&REPORT_INFO_LINE_TEXT] "The stream is {&stream_length} items long.");

    Report::new::<Self>(provider, &"Seek out of bounds")
      .with_info_line(&seek_text)
      .with_info_line(&length_text)
      .with_flag_line(LOW_LEVEL_ERROR)
      .apply(callback)
  }
}
