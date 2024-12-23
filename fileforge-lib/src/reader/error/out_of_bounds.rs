use fileforge_macros::text;

use crate::{
  diagnostic::node::reference::DiagnosticReference,
  error::{
    render::{
      buffer::cell::tag::builtin::report::{
        REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT,
      },
      builtin::{number::formatted_unsigned::FormattedUnsigned, text::Text},
    },
    report::{kind::ReportKind, note::ReportNote, Report},
    Error,
  },
  provider::out_of_bounds::SliceOutOfBoundsError,
};

#[derive(Clone)]
pub struct ReadOutOfBoundsError<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  pub read_offset: u64,
  pub read_size: Option<u64>,
  pub reader_size: u64,
  pub reader_diagnostic: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>
{
  pub fn from_slice_out_of_bounds_error(
    error: SliceOutOfBoundsError,
    diagnostic: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    ReadOutOfBoundsError {
      read_offset: error.read_offset,
      read_size: error.read_size,
      reader_size: error.provider_size,
      reader_diagnostic: diagnostic,
    }
  }
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, mut callback: Cb) {
    if !self.reader_diagnostic.family_exists() {
      let line = Text::new()
        .push("The diagnostic pool was too small to be able to load the diagnostics for this error. You are seeing a minified version with what available data exists.", &REPORT_FLAG_LINE_TEXT);

      let read_size_base_10 = self
        .read_size
        .map(|read_size| FormattedUnsigned::new(read_size).with_separator(3, ","));

      let read_offset_base_10 = FormattedUnsigned::new(self.read_offset).with_separator(3, ",");
      let read_offset_base_16 = FormattedUnsigned::new(self.read_offset)
        .with_base(16)
        .with_uppercase();

      let info = text!(
        { read_size_base_10.is_some() }
          [&REPORT_INFO_LINE_TEXT] "Failed to read {read_size_base_10.as_ref().unwrap()} bytes at {&read_offset_base_10} (0x{&read_offset_base_16}).",

        [&REPORT_INFO_LINE_TEXT] "Failed to read at {&read_offset_base_10} (0x{&read_offset_base_16}).",
      );

      return callback(
        Report::new::<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(
          ReportKind::Error,
          "Read Out Of Bounds",
        )
        .with_flag_line(&line)
        .unwrap()
        .with_info_line(&info)
        .unwrap(),
      );
    }

    let read_size_base_10 = self
      .read_size
      .map(|read_size| FormattedUnsigned::new(read_size).with_separator(3, ","));

    let read_offset_base_10 = FormattedUnsigned::new(self.read_offset).with_separator(3, ",");
    let read_offset_base_16 = FormattedUnsigned::new(self.read_offset)
      .with_base(16)
      .with_uppercase();

    let info = text!(
      { read_size_base_10.is_some() }
        [&REPORT_ERROR_TEXT] "Failed to read {read_size_base_10.as_ref().unwrap()} bytes at {&read_offset_base_10} (0x{&read_offset_base_16}).",

      [&REPORT_ERROR_TEXT] "Failed to read at {&read_offset_base_10} (0x{&read_offset_base_16}).",
    );

    return callback(
      Report::new::<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(
        ReportKind::Error,
        "Read Out Of Bounds",
      )
      .with_note(|| {
        ReportNote::new(&info)
          .with_tag(&REPORT_ERROR_TEXT)
          .with_unvalued_location(self.reader_diagnostic)
          .unwrap()
      })
      .unwrap(),
    );
  }
}
