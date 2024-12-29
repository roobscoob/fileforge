use fileforge_macros::text;

use crate::{
  diagnostic::{node::reference::DiagnosticReference, value::DiagnosticValue},
  error::{
    context::ErrorContext,
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_SYMBOL, REPORT_INFO_LINE_TEXT},
      builtin::number::formatted_unsigned::FormattedUnsigned,
    },
    report::{kind::ReportKind, location::ReportLocation, note::ReportNote, Report},
    FileforgeError,
  },
};

pub struct ReadOutOfBounds<'pool, const NODE_NAME_SIZE: usize> {
  pub read_offset: DiagnosticValue<'pool, u64, NODE_NAME_SIZE>,
  pub read_length: DiagnosticValue<'pool, u64, NODE_NAME_SIZE>,
  pub provider_size: DiagnosticValue<'pool, u64, NODE_NAME_SIZE>,
  pub container_dr: DiagnosticReference<'pool, NODE_NAME_SIZE>,
}

impl<'pool, const NODE_NAME_SIZE: usize> FileforgeError<NODE_NAME_SIZE> for ReadOutOfBounds<'pool, NODE_NAME_SIZE> {
  fn render_into_report(&self, mut callback: impl FnMut(Report<NODE_NAME_SIZE>) -> ()) {
    let context = ErrorContext::new()
      .with("read_offset", self.read_offset.reference())
      .with("read_length", self.read_length.reference())
      .with("provider_size", self.provider_size.reference())
      .with("container", self.container_dr);

    let read_length_base_10 = FormattedUnsigned::new(*self.read_length as u128).separator(3, ",");
    let read_length_base_16 = FormattedUnsigned::new(*self.read_length as u128).base(16).uppercase().prefix("0x");
    let read_offset_base_10 = FormattedUnsigned::new(*self.read_offset as u128).separator(3, ",");
    let read_offset_base_16 = FormattedUnsigned::new(*self.read_offset as u128).base(16).uppercase().prefix("0x");
    let container_size_base_10 = FormattedUnsigned::new(*self.provider_size as u128).separator(3, ",");

    let did_overflow = (*self.read_offset).checked_add(*self.read_length).is_none();

    let read_end_base_10 = FormattedUnsigned::new(*self.read_offset as u128 + *self.read_length as u128).separator(3, ",");
    let read_end_base_16 = FormattedUnsigned::new(*self.read_offset as u128 + *self.read_length as u128).base(16).uppercase().prefix("0x");

    let report_text = text!(
      { did_overflow }
        [&REPORT_ERROR_TEXT] "Failed to read {&read_length_base_10} bytes at {&read_offset_base_10} ({&read_offset_base_16}). The read end ({&read_end_base_16}) was beyond the 64-bit unsigned integer limit, causing an overflow.",

      [&REPORT_ERROR_TEXT] "Failed to read {&read_length_base_10} bytes at {&read_offset_base_10} ({&read_offset_base_16}). The read end ({&read_end_base_10}) was beyond the container's length, of {&container_size_base_10} bytes."
    );

    let mut report = Report::new::<Self>(ReportKind::Error, "Read out of Bounds")
      .with_error_context(&context)
      .with_flag_line(const_text!([&REPORT_FLAG_LINE_TEXT] "This is a low-level error, intended to be consumed by higher-level error handling code. This error is not intended to be displayed to the user. If you're seeing this error and *not* a library author, it may be confusing. Please report this error to the library author."))
      .unwrap()
      .with_note(|| {
        ReportNote::new(&report_text)
          .with_tag(&REPORT_ERROR_TEXT)
          .maybe_with_unvalued_location(context.get("container")).unwrap()
      }).unwrap();

    if let Some(reference) = context.get("read_offset") {
      let note = ReportNote::new(const_text!([&REPORT_INFO_LINE_TEXT] "The read offset was derived from here"))
        .with_tag(&REPORT_INFO_LINE_SYMBOL)
        .with_raw_location(ReportLocation {
          reference,
          value: Some(&read_offset_base_16),
        })
        .unwrap();

      report.add_note(note).unwrap();
    }

    if let Some(reference) = context.get("read_length") {
      let note = ReportNote::new(const_text!([&REPORT_INFO_LINE_TEXT] "The read length was derived from here"))
        .with_tag(&REPORT_INFO_LINE_SYMBOL)
        .with_raw_location(ReportLocation {
          reference,
          value: Some(&read_length_base_16),
        })
        .unwrap();

      report.add_note(note).unwrap();
    }

    if !did_overflow {
      if let Some(reference) = context.get("provider_size") {
        let note = ReportNote::new(const_text!([&REPORT_INFO_LINE_TEXT] "The container's size was derived from here"))
          .with_tag(&REPORT_INFO_LINE_SYMBOL)
          .with_raw_location(ReportLocation {
            reference,
            value: Some(&read_length_base_16),
          })
          .unwrap();

        report.add_note(note).unwrap();
      }
    }

    callback(report);
  }
}
