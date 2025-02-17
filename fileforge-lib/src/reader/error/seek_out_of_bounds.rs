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

pub enum SeekOffset {
  Underflow { base_offset: u64, subtract: u64 },
  Overflowed { base_offset: u64, add: u64 },
  InBounds(u64),
}

impl SeekOffset {
  pub fn value(&self) -> FormattedUnsigned<'static> {
    match self {
      Self::InBounds(v) => FormattedUnsigned::new(*v as u128),
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

pub struct SeekOutOfBounds<'pool, const NODE_NAME_SIZE: usize> {
  pub seek_offset: SeekOffset,
  pub provider_size: DiagnosticValue<'pool, u64, NODE_NAME_SIZE>,
  pub container_dr: Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>,
}

impl<'pool, const NODE_NAME_SIZE: usize> FileforgeError<NODE_NAME_SIZE> for SeekOutOfBounds<'pool, NODE_NAME_SIZE> {
  fn render_into_report(&self, mut callback: impl FnMut(Report<'static, '_, '_, NODE_NAME_SIZE>) -> ()) {
    let context = ErrorContext::new().with("provider_size", self.provider_size.reference()).with("container", self.container_dr);

    let seek_offset_base_10 = self.seek_offset.value().separator(3, ",");
    let seek_offset_base_16 = self.seek_offset.value().base(16).uppercase();
    let container_size_base_10 = FormattedUnsigned::new(*self.provider_size as u128).separator(3, ",");
    let container_size_base_16 = FormattedUnsigned::new(*self.provider_size as u128).base(16).uppercase().prefix("0x");

    let report_text = text!(
      { self.seek_offset.did_overflow() }
        [&REPORT_ERROR_TEXT] "Failed to seek to {&seek_offset_base_10} (0x{&seek_offset_base_16}). The seek point was beyond the 64-bit unsigned integer limit, causing an overflow.",

      { self.seek_offset.did_underflow() }
        [&REPORT_ERROR_TEXT] "Failed to seek to {&seek_offset_base_10}. The seek point underflowed the 64 bit unsigned integer minimum (0).",

      [&REPORT_ERROR_TEXT] "Failed to seek to {&seek_offset_base_10} (0x{&seek_offset_base_16}). The seek point was beyond the container's length, of {&container_size_base_10} bytes."
    );

    let mut report = Report::new::<Self>(ReportKind::Error, "Seek out of Bounds")
      .with_error_context(&context)
      .with_flag_line(const_text!([&REPORT_FLAG_LINE_TEXT] "This is a low-level error, intended to be consumed by higher-level error handling code. This error is not intended to be displayed to the user. If you're seeing this error and *not* a library author, it may be confusing. Please report this error to the library author."))
      .unwrap();

    // .with_note(|| {
    //   ReportNote::new(&report_text)
    //     .with_tag(&REPORT_ERROR_TEXT)
    //     .maybe_with_unvalued_location().unwrap()
    // }).unwrap();

    if let Some(location) = context.get("container") {
      report
        .add_note(ReportNote::new(&report_text).with_tag(&REPORT_ERROR_TEXT).with_unvalued_location(location).unwrap())
        .unwrap()
    } else {
      report.add_info_line(&report_text).unwrap()
    }

    if !self.seek_offset.did_overflow() && !self.seek_offset.did_underflow() {
      if let Some(reference) = context.get("provider_size") {
        let note = ReportNote::new(const_text!([&REPORT_INFO_LINE_TEXT] "The container's size was derived from here"))
          .with_tag(&REPORT_INFO_LINE_SYMBOL)
          .with_raw_location(ReportLocation {
            reference,
            value: Some(&container_size_base_16),
          })
          .unwrap();

        report.add_note(note).unwrap();
      }
    }

    callback(report);
  }
}
