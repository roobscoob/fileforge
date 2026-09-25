use fileforge_macros::{story, text};
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use crate::binary_reader::error::common::LOW_LEVEL_ERROR;
use crate::error::render::buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT};
use crate::error::render::builtin::number::formatted_unsigned::FormattedUnsigned;
use crate::provider::error::out_of_bounds::OutOfBoundsError;

#[story("read past the end", StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 })]
#[derive(Debug)]
pub struct StreamExhaustedError {
  pub stream_length: u64,
  pub read_length: u64,
  pub read_offset: u64,
}

impl From<OutOfBoundsError> for Option<StreamExhaustedError> {
  fn from(value: OutOfBoundsError) -> Self {
    value.read_length.map(|read_length| StreamExhaustedError {
      stream_length: value.provider_size,
      read_length,
      read_offset: value.read_offset,
    })
  }
}

impl StreamExhaustedError {
  pub fn assert(stream_length: u64, read_offset: u64, read_length: u64) -> Result<(), Self> {
    let read_end = read_offset.checked_add(read_length).ok_or(Self {
      read_offset,
      read_length,
      stream_length,
    })?;

    if read_end > stream_length {
      Err(Self {
        read_offset,
        read_length,
        stream_length,
      })
    } else {
      Ok(())
    }
  }
}

impl FileforgeError for StreamExhaustedError {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    let past_end = (self.read_offset as u128 + self.read_length as u128).saturating_sub(self.stream_length as u128);

    let read_length = FormattedUnsigned::new(self.read_length as u128).separator(3, ",");
    let read_offset = FormattedUnsigned::new(self.read_offset as u128).base(16).uppercase().prefix("0x");
    let past_end = FormattedUnsigned::new(past_end).separator(3, ",");
    let stream_length = FormattedUnsigned::new(self.stream_length as u128).separator(3, ",");

    let read_text = text!(
      { self.read_length == 1 }
        [&REPORT_ERROR_TEXT] "Reading 1 item at {&read_offset} goes past the end of the stream.",

      [&REPORT_ERROR_TEXT] "Reading {&read_length} items at {&read_offset} goes {&past_end} past the end of the stream."
    );
    let length_text = text!([&REPORT_INFO_LINE_TEXT] "The stream is {&stream_length} items long.");

    Report::new::<Self>(provider, &"Stream exhausted")
      .with_info_line(&read_text)
      .with_info_line(&length_text)
      .with_flag_line(LOW_LEVEL_ERROR)
      .apply(callback)
  }
}
