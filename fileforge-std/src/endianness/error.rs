use fileforge_lib::{
  diagnostic::node::tagged_reference::TaggedDiagnosticReference,
  error::{
    render::{
      buffer::cell::tag::builtin::report::{
        REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT,
      },
      builtin::text::Text,
    },
    report::{kind::ReportKind, note::ReportNote, Report},
    Error,
  },
};

use crate::magic::Magic;

use super::EndiannessMarker;

pub struct EndiannessMarkerError<
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  const ENDIANNESS_SIZE: usize,
> {
  pub expected: EndiannessMarker<ENDIANNESS_SIZE>,
  pub actual: TaggedDiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE, Magic<ENDIANNESS_SIZE>>,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const ENDIANNESS_SIZE: usize>
  EndiannessMarkerError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, ENDIANNESS_SIZE>
{
  fn print_with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(
    &self,
    mut callback: Cb,
  ) {
    let inverted = self.expected.inverse_clone();

    let magic_info = Text::new().push(
      "A broken endianness marker indicates the slice is entirely invalid or corrupt.",
      &REPORT_FLAG_LINE_TEXT,
    );

    let info = Text::new()
      .push("Expected ", &REPORT_INFO_LINE_TEXT)
      .with(&self.expected)
      .push(" or ", &REPORT_INFO_LINE_TEXT)
      .with(&inverted)
      .push(" got ", &REPORT_INFO_LINE_TEXT)
      .with(self.actual.value());

    let note = Text::new()
      .push("Expected ", &REPORT_ERROR_TEXT)
      .with(&self.expected)
      .push(" or ", &REPORT_ERROR_TEXT)
      .with(&inverted);

    callback(
      Report::new::<EndiannessMarkerError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, ENDIANNESS_SIZE>>(
        ReportKind::Error,
        "Invalid Endianness",
      )
      .with_info_line(&info)
      .unwrap()
      .with_flag_line(&magic_info)
      .unwrap()
      .with_note(|| {
        ReportNote::new(&note)
          .with_tag(&REPORT_ERROR_TEXT)
          .with_location(self.actual.reference(), self.actual.value())
          .unwrap()
      })
      .unwrap(),
    )
  }

  fn print_with_minified_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(
    &self,
    mut callback: Cb,
  ) {
    let line = Text::new()
      .push("The diagnostic pool was too small to be able to load the diagnostics for this error. You are seeing a minified version with what available data exists.", &REPORT_FLAG_LINE_TEXT);

    let magic_info = Text::new().push(
      "A broken endianness marker indicates the slice is entirely invalid or corrupt.",
      &REPORT_FLAG_LINE_TEXT,
    );

    let inverted = self.expected.inverse_clone();

    let info = Text::new()
      .push("Expected ", &REPORT_INFO_LINE_TEXT)
      .with(&self.expected)
      .push(" or ", &REPORT_INFO_LINE_TEXT)
      .with(&inverted)
      .push(" got ", &REPORT_INFO_LINE_TEXT)
      .with(self.actual.value());

    callback(
      Report::new::<EndiannessMarkerError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, ENDIANNESS_SIZE>>(
        ReportKind::Error,
        "Invalid Endianness",
      )
      .with_flag_line(&line)
      .unwrap()
      .with_flag_line(&magic_info)
      .unwrap()
      .with_info_line(&info)
      .unwrap(),
    );
  }
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const ENDIANNESS_SIZE: usize>
  Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for EndiannessMarkerError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, ENDIANNESS_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    if !self.actual.reference().family_exists() {
      self.print_with_minified_report(callback)
    } else {
      self.print_with_report(callback)
    }
  }
}
