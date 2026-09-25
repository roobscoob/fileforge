use fileforge_macros::story;
use fileforge::{
  binary_reader::error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    ext::annotations::annotated::Annotated,
    render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::r#const::ConstText},
    report::Report,
    FileforgeError,
  },
  stream::error::user_read::UserReadError,
};

use crate::byte_order_mark::error::invalid::ByteOrderMarkInvalid;

pub mod invalid;

#[story("read failed", ByteOrderMarkError::Failed(read_exhausted::<[u8; 2], StoryUserError>(dr!("archive.sarc" @ 0..7), 6, DiagnosticValue(7, None))))]
#[story("invalid", ByteOrderMarkError::<StoryUserError>::Invalid(ByteOrderMarkInvalid {
  expected: crate::byte_order_mark::ByteOrderMark::from_bytes(fileforge::binary_reader::endianness::Endianness::BigEndian, [0xFE, 0xFF]),
  actual: dv!("archive.sarc" / "ByteOrderMark" @ 6..8 [[0x12, 0x34]]),
}))]
pub enum ByteOrderMarkError<'pool, U: UserReadError> {
  Failed(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  Invalid(ByteOrderMarkInvalid<'pool>),
}

const READING_BYTE_ORDER_MARK: ConstText = ConstText::new("This happened while reading a byte order mark.", &REPORT_INFO_LINE_TEXT);

impl<'pool, U: UserReadError> FileforgeError for ByteOrderMarkError<'pool, U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Failed(read) => read.error().render_into_report(provider, |report| report.with_info_line(&READING_BYTE_ORDER_MARK).apply(callback)),
      Self::Invalid(inner) => inner.render_into_report(provider, callback),
    }
  }
}

impl<'pool, U: UserReadError> From<Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>> for ByteOrderMarkError<'pool, U> {
  fn from(value: Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>) -> Self {
    Self::Failed(value)
  }
}

impl<'pool, U: UserReadError> From<ByteOrderMarkInvalid<'pool>> for ByteOrderMarkError<'pool, U> {
  fn from(value: ByteOrderMarkInvalid<'pool>) -> Self {
    Self::Invalid(value)
  }
}
