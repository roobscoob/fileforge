use fileforge_macros::text;

use crate::{
  binary_reader::error::common::{SeekOffset, LOW_LEVEL_ERROR},
  diagnostic::{node::reference::DiagnosticReference, pool::DiagnosticPoolProvider, value::DiagnosticValue},
  error::{
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_SYMBOL, REPORT_INFO_LINE_TEXT},
      builtin::{number::formatted_unsigned::FormattedExt, text::r#const::ConstText},
    },
    report::Report,
    FileforgeError,
  },
};

pub struct SeekOutOfBounds<'pool> {
  pub seek_offset: SeekOffset,
  pub provider_size: DiagnosticValue<'pool, u64>,
  pub container_dr: Option<DiagnosticReference<'pool>>,
}

const DERIVED_TEXT: &'static ConstText = const_text!([&REPORT_INFO_LINE_TEXT] "The container's size was derived from here");

impl<'pool> FileforgeError for SeekOutOfBounds<'pool> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    let seek_offset_base_10 = self.seek_offset.format().separator(3, ",");
    let seek_offset_base_16 = self.seek_offset.format().base(16).uppercase();
    let container_size_base_10 = self.provider_size.format().separator(3, ",");
    let provider_size = self.provider_size.map(|v| v.format().base(16).uppercase().prefix("0x"));
    let within_bounds = !self.seek_offset.did_overflow() && !self.seek_offset.did_underflow();

    let report_text = text!(
      { self.seek_offset.did_overflow() }
        [&REPORT_ERROR_TEXT] "Failed to seek to {&seek_offset_base_10} (0x{&seek_offset_base_16}). The seek point was beyond the 64-bit unsigned integer limit, causing an overflow.",

      { self.seek_offset.did_underflow() }
        [&REPORT_ERROR_TEXT] "Failed to seek to {&seek_offset_base_10}. The seek point underflowed the 64 bit unsigned integer minimum (0).",

      [&REPORT_ERROR_TEXT] "Failed to seek to {&seek_offset_base_10} (0x{&seek_offset_base_16}). The seek point was beyond the container's length, of {&container_size_base_10} bytes."
    );

    Report::new::<Self>(provider, &"Seek out of Bounds")
      .with_flag_line(LOW_LEVEL_ERROR)
      .with_error_context()
      .with_opt_context("container", self.container_dr)
      .with_contextual_note_or_info("container", &report_text, |n| n.with_tag(&REPORT_ERROR_TEXT))
      .with_context("provider_size", &provider_size)
      .with_contextual_note_if(within_bounds, "provider_size", DERIVED_TEXT, |v| v.with_tag(&REPORT_INFO_LINE_SYMBOL))
      .finalize_context()
      .apply(callback);
  }
}
