use fileforge_lib::{diagnostic::node::tagged_reference::TaggedDiagnosticReference, error::{render::{buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT}, builtin::text::Text}, report::{kind::ReportKind, note::ReportNote, Report}, Error}};

use super::Magic;

pub struct MagicError<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const MAGIC_SIZE: usize> {
  pub expected: Magic<MAGIC_SIZE>,
  pub actual: TaggedDiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE, Magic<MAGIC_SIZE>>
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const MAGIC_SIZE: usize> MagicError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE> {
  fn print_with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, mut callback: Cb) {
    let magic_info = Text::new()
      .push("A broken magic indicates the slice is entirely invalid or corrupt.", &REPORT_FLAG_LINE_TEXT);

    let info = Text::new()
      .push("Expected ", &REPORT_INFO_LINE_TEXT)
      .with(&self.expected)
      .push(" got ", &REPORT_INFO_LINE_TEXT)
      .with(self.actual.value());

    let note = Text::new()
      .push("Expected ", &REPORT_ERROR_TEXT)
      .with(&self.expected);

    callback(Report::new::<MagicError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE>>(ReportKind::Error, "Invalid Magic")
      .with_info_line(&info).unwrap()
      .with_flag_line(&magic_info).unwrap()
      .with_note(|| {
        ReportNote::new(&note)
          .with_tag(&REPORT_ERROR_TEXT)
          .with_location(self.actual.reference(), self.actual.value()).unwrap()
      }).unwrap())
  }

  fn print_with_minified_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, mut callback: Cb) {
    let line = Text::new()
      .push("The diagnostic pool was too small to be able to load the diagnostics for this error. You are seeing a minified version with what available data exists.", &REPORT_FLAG_LINE_TEXT);

    let magic_info = Text::new()
      .push("A broken magic indicates the slice is entirely invalid or corrupt.", &REPORT_FLAG_LINE_TEXT);

    let info = Text::new()
      .push("Expected ", &REPORT_INFO_LINE_TEXT)
      .with(&self.expected)
      .push(" got ", &REPORT_INFO_LINE_TEXT)
      .with(self.actual.value());

    callback(Report::new::<MagicError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE>>(ReportKind::Error, "Invalid Magic")
      .with_flag_line(&line).unwrap()
      .with_info_line(&info).unwrap()
      .with_flag_line(&magic_info).unwrap());
  }
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const MAGIC_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for MagicError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE> {
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    if !self.actual.reference().family_exists() {
      self.print_with_minified_report(callback)
    } else {
      self.print_with_report(callback)
    }
  }
}