use core::ffi::FromBytesUntilNulError;

use fileforge_macros::text;

use fileforge_lib::{
  diagnostic::node::reference::DiagnosticReference,
  error::{
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT},
      builtin::byte_display::ByteDisplay,
    },
    report::{kind::ReportKind, note::ReportNote, Report},
    Error,
  },
  provider::error::ProviderError,
  reader::error::{
    underlying_provider_error::UnderlyingProviderError,
    underlying_provider_read::UnderlyingProviderReadError,
  },
};

use super::get_length::StringTableNotLargeEnough;

pub enum GetError<
  'pool,
  PE: ProviderError,
  SE: ProviderError,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  const BYTE_DISPLAY_SIZE: usize,
> {
  UnderlyingProviderError(UnderlyingProviderError<'pool, PE, SE, DIAGNOSTIC_NODE_NAME_SIZE>),
  IndexOutOfBounds {
    requested_index: u32,
    length_dr: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
    length_value: u32,
  },
  CStrError(
    FromBytesUntilNulError,
    DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
    ByteDisplay<BYTE_DISPLAY_SIZE>,
  ),
  NotLargeEnough(StringTableNotLargeEnough<SE>),
}

impl<
    'pool,
    PE: ProviderError,
    SE: ProviderError,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
    const BYTE_DISPLAY_SIZE: usize,
  > Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for GetError<'pool, PE, SE, DIAGNOSTIC_NODE_NAME_SIZE, BYTE_DISPLAY_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, mut callback: Cb) {
    match self {
      Self::UnderlyingProviderError(upe) => upe.with_report(callback),
      Self::NotLargeEnough(nle) => nle.with_report(callback),
      Self::CStrError(_, dr, disp) => {
        let minified = &text!([&REPORT_FLAG_LINE_TEXT] "The diagnostic pool was too small to be able to load the diagnostics for this error. You are seeing a minified version with what available data exists.");
        let note = &text!([&REPORT_ERROR_TEXT] "CStr does not contain a NUL (0x00)");
        let mut report = Report::new::<FromBytesUntilNulError>(
          ReportKind::Error,
          "CStr does not contain a NUL (0x00)",
        );

        if dr.family_exists() {
          report = report
            .with_note(|| ReportNote::new(note).with_location(*dr, disp).unwrap())
            .unwrap()
        } else {
          report = report.with_flag_line(minified).unwrap()
        }

        callback(report);
      }
      Self::IndexOutOfBounds {
        requested_index,
        length_dr,
        length_value,
      } => {
        todo!();
      }
    }
  }
}
