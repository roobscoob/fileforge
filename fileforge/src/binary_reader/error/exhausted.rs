use fileforge_macros::{story, text};

use crate::{
  binary_reader::error::common::{ExhaustedType, LOW_LEVEL_ERROR},
  diagnostic::{node::reference::DiagnosticReference, pool::DiagnosticPoolProvider, value::DiagnosticValue},
  error::{
    render::{
      buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT,
      builtin::number::formatted_unsigned::{FormattedExt, FormattedUnsigned},
    },
    report::Report,
    FileforgeError,
  },
};

#[story("read past the end, no diagnostics", crate::binary_reader::error::exhausted::ReaderExhaustedError {
  container: None,
  length: DiagnosticValue(4, None),
  offset: 0,
  stream_length: DiagnosticValue(2, None),
  t: crate::binary_reader::error::common::Read,
})]
#[story("read one byte past the end", crate::binary_reader::error::exhausted::ReaderExhaustedError {
  container: None,
  length: DiagnosticValue(4, None),
  offset: 4,
  stream_length: DiagnosticValue(7, None),
  t: crate::binary_reader::error::common::Read,
})]
#[story("read past the end, with diagnostics", crate::binary_reader::error::exhausted::ReaderExhaustedError {
  container: dr!("save.bin" @ 0..24),
  length: dv!("save.bin" / "header.entry_size" @ 4..8 [16]),
  offset: 16,
  stream_length: dv!("save.bin" / "header.file_size" @ 0..4 [24]),
  t: crate::binary_reader::error::common::Read,
})]
#[story("write past the end, with diagnostics", crate::binary_reader::error::exhausted::ReaderExhaustedError {
  container: dr!("save.bin" @ 0..8),
  length: DiagnosticValue(4, None),
  offset: 6,
  stream_length: dv!("save.bin" / "header.file_size" @ 0..4 [8]),
  t: crate::binary_reader::error::common::Write,
})]
pub struct ReaderExhaustedError<'pool, T: ExhaustedType> {
  pub container: Option<DiagnosticReference<'pool>>,
  pub length: DiagnosticValue<'pool, u64>,
  pub offset: u64,
  pub stream_length: DiagnosticValue<'pool, u64>,
  pub t: T,
}

impl<'pool, T: ExhaustedType> FileforgeError for ReaderExhaustedError<'pool, T> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'a, 'b> FnOnce(Report<'a, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    let container_size = self.stream_length.map(|v| v.format().base(16).uppercase().prefix("0x"));
    let length = self.length.map(|v| v.format().base(16).uppercase().prefix("0x"));

    let message = self.t.message(&REPORT_INFO_LINE_TEXT);

    let overflow_size = (self.offset as u128 + *self.length as u128) - *self.stream_length as u128;
    let overflow_size_base_10 = FormattedUnsigned::new(overflow_size).separator(3, ",");
    let report_text = text!(
      {overflow_size == 1}
        [&REPORT_INFO_LINE_TEXT] "Attempted to {&message} 1 byte outside the range provided by the provider.",

      [&REPORT_INFO_LINE_TEXT] "Attempted to {&message} {&overflow_size_base_10} bytes outside the range provided by the provider."
    );

    let container_size_base_10 = self.stream_length.format().separator(3, ",");
    let provider_text = text!(
      {self.stream_length.reference().is_some()}
        [&REPORT_INFO_LINE_TEXT] "Here, the provider supplied {&container_size_base_10} ({&*container_size}) bytes",

      [&REPORT_INFO_LINE_TEXT] "The provider supplied {&container_size_base_10} ({&*container_size}) bytes",
    );

    let length_base_10 = FormattedUnsigned::from(self.length).separator(3, ",");
    let offset_base_16 = FormattedUnsigned::from(self.offset).base(16).uppercase().prefix("0x");
    let read_text = text!(
      {self.length.reference().is_some()}
        [&REPORT_INFO_LINE_TEXT] "The {&message} requested {&length_base_10} ({&*length}) bytes at offset {&offset_base_16}. This is where the {&message} length originated from.",

      [&REPORT_INFO_LINE_TEXT] "The {&message} requested {&length_base_10} ({&*length}) bytes at offset {&offset_base_16}"
    );

    Report::new::<Self>(provider, &"Reader Exhausted")
      .with_error_context()
      .with_context("length", &length)
      .with_context("stream_length", &container_size)
      .with_opt_context("container", self.container)
      .with_contextual_note("container", const_text!([&REPORT_INFO_LINE_TEXT] "This is the container"), |v| v.with_tag(&REPORT_INFO_LINE_TEXT))
      .with_contextual_note_or_info("stream_length", &provider_text, |v| v.with_tag(&REPORT_INFO_LINE_TEXT))
      .with_contextual_note_or_info("length", &read_text, |v| v.with_tag(&REPORT_INFO_LINE_TEXT))
      .finalize_context()
      .with_flag_line(LOW_LEVEL_ERROR)
      .with_info_line(&report_text)
      .apply(callback);
  }
}
