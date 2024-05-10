use crate::{diagnostic::node::tagged_reference::TaggedDiagnosticReference, error::{render::{buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT}, builtin::text::Text}, report::{kind::ReportKind, note::ReportNote, Report}, Error}};

use super::Magic;

pub struct MagicError<'pool_lifetime, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const MAGIC_SIZE: usize> {
  pub expected: Magic<MAGIC_SIZE>,
  pub actual: TaggedDiagnosticReference<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, Magic<MAGIC_SIZE>>
}

impl<'pool_lifetime, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const MAGIC_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for MagicError<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE> {
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, mut callback: Cb) {
    if !self.actual.reference().family_exists() {
      let line = Text::new()
        .push("The diagnostic pool was too small to be able to load the diagnostics for this error. You are seeing a minified version with what available data exists.", &REPORT_FLAG_LINE_TEXT);

      let info = Text::new()
        .push("Expected ", &REPORT_INFO_LINE_TEXT)
        .with(&self.expected)
        .push(" got ", &REPORT_INFO_LINE_TEXT)
        .with(self.actual.value());

      callback(Report::new::<MagicError<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE>>(ReportKind::Error, "Invalid Magic")
        .with_flag_line(&line).unwrap()
        .with_info_line(&info).unwrap());
    }

    let info = Text::new()
      .push("Expected ", &REPORT_INFO_LINE_TEXT)
      .with(&self.expected)
      .push(" got ", &REPORT_INFO_LINE_TEXT)
      .with(self.actual.value());

    let note = Text::new()
      .push("Expected ", &REPORT_ERROR_TEXT)
      .with(&self.expected);

    callback(Report::new::<MagicError<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE>>(ReportKind::Error, "Invalid Magic")
      .with_info_line(&info).unwrap()
      .with_note(|| {
        ReportNote::new(&note)
          .with_tag(&REPORT_ERROR_TEXT)
          .with_location(self.actual.reference(), self.actual.value()).unwrap()
      }).unwrap())
  }
}