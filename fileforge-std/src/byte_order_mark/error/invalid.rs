use fileforge::{
  binary_reader::endianness::Endianness,
  diagnostic::{node::reference::DiagnosticReference, pool::DiagnosticPoolProvider, value::DiagnosticValue},
  error::{
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT},
      builtin::bytes::Bytes,
    },
    report::{note::ReportNote, Report},
    FileforgeError,
  },
};
use fileforge_macros::{story, text};

use super::super::ByteOrderMark;
use crate::magic::WRONG_FORMAT;

#[story("neither byte order, with diagnostics", ByteOrderMarkInvalid {
  expected: ByteOrderMark::from_bytes(Endianness::BigEndian, [0xFE, 0xFF]),
  actual: dv!("archive.sarc" / "ByteOrderMark" @ 6..8 [[0x12, 0x34]]),
})]
#[story("neither byte order, no diagnostics", ByteOrderMarkInvalid {
  expected: ByteOrderMark::from_bytes(Endianness::BigEndian, [0xFE, 0xFF]),
  actual: DiagnosticValue([0x12, 0x34], None),
})]
#[story("neither byte order, printable bytes", ByteOrderMarkInvalid {
  expected: ByteOrderMark::from_bytes(Endianness::BigEndian, *b"BY"),
  actual: dv!("data.byml" / "ByteOrderMark" @ 0..2 [*b"XX"]),
})]
pub struct ByteOrderMarkInvalid<'pool> {
  pub expected: ByteOrderMark,
  pub actual: DiagnosticValue<'pool, [u8; 2]>,
}

impl<'pool> ByteOrderMarkInvalid<'pool> {
  pub fn assert(expected: ByteOrderMark, actual: [u8; 2], get_dr: impl FnOnce() -> Option<DiagnosticReference<'pool>>) -> Result<Endianness, Self> {
    if expected.bytes() == actual {
      return Ok(expected.endianness());
    } else if expected.swap().bytes() == actual {
      return Ok(expected.endianness().swap());
    };

    Err(ByteOrderMarkInvalid {
      expected,
      actual: DiagnosticValue(actual, get_dr()),
    })
  }
}

impl<'pool> FileforgeError for ByteOrderMarkInvalid<'pool> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    let (big_endian, little_endian) = match self.expected.endianness() {
      Endianness::BigEndian => (self.expected, self.expected.swap()),
      Endianness::LittleEndian => (self.expected.swap(), self.expected),
    };

    let found = Bytes(*self.actual);
    let big_endian = Bytes(big_endian.bytes());
    let little_endian = Bytes(little_endian.bytes());

    let found_text = text!([&REPORT_ERROR_TEXT] "Found {&found}, which is not a valid byte order mark.");
    let big_endian_text = text!([&REPORT_INFO_LINE_TEXT] "Big-endian data has {&big_endian} here.");
    let little_endian_text = text!([&REPORT_INFO_LINE_TEXT] "Little-endian data has {&little_endian} here.");
    let note_text = text!([&REPORT_INFO_LINE_TEXT] "Expected {&big_endian} or {&little_endian} here");

    let location = self.actual.map(Bytes);

    let mut report = Report::new::<Self>(provider, &"Invalid byte order mark")
      .with_info_line(&found_text)
      .with_info_line(&big_endian_text)
      .with_info_line(&little_endian_text)
      .with_flag_line(&WRONG_FORMAT);

    if location.reference().is_some() {
      report.add_note(ReportNote::new(&note_text).with_location(&location).with_tag(&REPORT_INFO_LINE_TEXT));
    }

    report.apply(callback);
  }
}
