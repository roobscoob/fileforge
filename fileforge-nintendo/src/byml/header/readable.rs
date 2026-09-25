use fileforge::{
  binary_reader::{
    endianness::Endianness,
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    ext::annotations::annotated::Annotated,
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT},
      builtin::bytes::Bytes,
    },
    report::{note::ReportNote, Report},
    FileforgeError,
  },
  stream::{error::user_read::UserReadError, ReadableStream},
};
use fileforge_macros::{story, text};
use fileforge_std::{
  byte_order_mark::{
    error::{invalid::ByteOrderMarkInvalid, ByteOrderMarkError},
    ByteOrderMark,
  },
  magic::WRONG_FORMAT,
};

use crate::report::{render_field_read, Field};

use crate::byml::header::BymlHeaderConfig;

use super::BymlHeader;

pub const BYML_BOM: ByteOrderMark = ByteOrderMark::from_byte_ref(Endianness::BigEndian, b"BY");

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for BymlHeader {
  type Error = BymlHeaderReadError<'pool, S::ReadError>;
  type Argument = BymlHeaderConfig;

  async fn read(reader: &mut BinaryReader<'pool, S>, config: Self::Argument) -> Result<Self, Self::Error> {
    let bom = reader.read_with::<ByteOrderMark>(BYML_BOM).await.map_err(|e| BymlHeaderReadError::ByteOrderMark(e))?;

    reader.set_endianness(bom.endianness());

    Ok(BymlHeader {
      config,
      endianness: bom.endianness(),
      version: reader.get().await.map_err(|e| BymlHeaderReadError::Version(e))?,
      key_table_offset: reader.get().await.map_err(|e| BymlHeaderReadError::KeyTableOffset(e))?,
      string_table_offset: reader.get().await.map_err(|e| BymlHeaderReadError::StringTableOffset(e))?,
      binary_data_table_offset: if config.feat_binary_data_table {
        reader.get().await.map_err(|e| BymlHeaderReadError::BinaryDataTableOffset(e))?
      } else {
        0
      },
      root_data_offset: reader.get().await.map_err(|e| BymlHeaderReadError::RootDataOffset(e))?,
    })
  }
}

#[story("file ends inside the byte order mark", BymlHeaderReadError::ByteOrderMark(fileforge_std::byte_order_mark::error::ByteOrderMarkError::Failed(read_exhausted::<[u8; 2], StoryUserError>(dr!("data.byml" @ 0..1), 0, DiagnosticValue(1, None)))))]
#[story("invalid byte order mark", BymlHeaderReadError::<StoryUserError>::ByteOrderMark(fileforge_std::byte_order_mark::error::ByteOrderMarkError::Invalid(fileforge_std::byte_order_mark::error::invalid::ByteOrderMarkInvalid {
  expected: fileforge_std::byte_order_mark::ByteOrderMark::from_bytes(fileforge::binary_reader::endianness::Endianness::BigEndian, *b"BY"),
  actual: dv!("data.byml" / "ByteOrderMark" @ 0..2 [*b"XX"]),
})))]
#[story("file ends inside the version", BymlHeaderReadError::Version(read_exhausted::<u16, StoryUserError>(dr!("data.byml" @ 0..3), 2, DiagnosticValue(3, None))))]
#[story("file ends inside the key table offset", BymlHeaderReadError::KeyTableOffset(read_exhausted::<u32, StoryUserError>(dr!("data.byml" @ 0..6), 4, DiagnosticValue(6, None))))]
#[story("file ends inside the string table offset", BymlHeaderReadError::StringTableOffset(read_exhausted::<u32, StoryUserError>(dr!("data.byml" @ 0..10), 8, DiagnosticValue(10, None))))]
#[story("file ends inside the binary data table offset", BymlHeaderReadError::BinaryDataTableOffset(read_exhausted::<u32, StoryUserError>(dr!("data.byml" @ 0..14), 12, DiagnosticValue(14, None))))]
#[story("file ends inside the root data offset", BymlHeaderReadError::RootDataOffset(read_exhausted::<u32, StoryUserError>(dr!("data.byml" @ 0..18), 16, DiagnosticValue(18, None))))]
#[story("file ends right before the key table offset", BymlHeaderReadError::KeyTableOffset(read_exhausted::<u32, StoryUserError>(dr!("data.byml" @ 0..4), 4, DiagnosticValue(4, None))))]
#[story("file ends inside the key table offset, no diagnostics", BymlHeaderReadError::KeyTableOffset(read_exhausted::<u32, StoryUserError>(None, 4, DiagnosticValue(6, None))))]
#[story("file inside an archive ends inside the key table offset", BymlHeaderReadError::KeyTableOffset(read_exhausted::<u32, StoryUserError>(
  dr!("archive.sarc" / "Bed.byml" @ 64..70),
  4,
  dv!("archive.sarc" / "SFAT entry 0" @ 32..48 / "data end offset" @ 12..16 [6]),
)))]
#[story("stream failed while reading the key table offset", BymlHeaderReadError::KeyTableOffset(read_failed::<u32, StoryUserError>(StoryUserError)))]
pub enum BymlHeaderReadError<'pool, U: UserReadError> {
  ByteOrderMark(ByteOrderMarkError<'pool, U>),
  Version(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  KeyTableOffset(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  StringTableOffset(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  BinaryDataTableOffset(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  RootDataOffset(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
}

impl<'pool, U: UserReadError> FileforgeError for BymlHeaderReadError<'pool, U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    let field = |name, kind| Field { structure: "BYML header", name, kind };

    match self {
      Self::ByteOrderMark(ByteOrderMarkError::Failed(read)) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("byte order mark", "2 bytes"), provider, callback),
      Self::ByteOrderMark(ByteOrderMarkError::Invalid(invalid)) => render_not_byml::<U, P, ITEM_NAME_SIZE>(invalid, provider, callback),
      Self::Version(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("version", "a u16"), provider, callback),
      Self::KeyTableOffset(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("key table offset", "a u32"), provider, callback),
      Self::StringTableOffset(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("string table offset", "a u32"), provider, callback),
      Self::BinaryDataTableOffset(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("binary data table offset", "a u32"), provider, callback),
      Self::RootDataOffset(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("root node offset", "a u32"), provider, callback),
    }
  }
}

/// A BYML file's byte order mark ("BY" or "YB") is also its magic, so a wrong one means the data
/// isn't BYML at all.
fn render_not_byml<'pool, U: UserReadError, P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
  invalid: &ByteOrderMarkInvalid<'pool>,
  provider: P,
  callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
) {
  let (big_endian, little_endian) = match invalid.expected.endianness() {
    Endianness::BigEndian => (invalid.expected, invalid.expected.swap()),
    Endianness::LittleEndian => (invalid.expected.swap(), invalid.expected),
  };

  let found = Bytes(*invalid.actual);
  let big_endian = Bytes(big_endian.bytes());
  let little_endian = Bytes(little_endian.bytes());

  let found_text = text!([&REPORT_ERROR_TEXT] "Found {&found}.");
  let big_endian_text = text!([&REPORT_INFO_LINE_TEXT] "Big-endian BYML files start with {&big_endian}.");
  let little_endian_text = text!([&REPORT_INFO_LINE_TEXT] "Little-endian BYML files start with {&little_endian}.");
  let note_text = text!([&REPORT_INFO_LINE_TEXT] "Expected {&big_endian} or {&little_endian} here");

  let location = invalid.actual.map(Bytes);

  let mut report = Report::new::<BymlHeaderReadError<'pool, U>>(provider, &"Not a BYML file")
    .with_info_line(&found_text)
    .with_info_line(&big_endian_text)
    .with_info_line(&little_endian_text)
    .with_flag_line(&WRONG_FORMAT);

  if location.reference().is_some() {
    report.add_note(ReportNote::new(&note_text).with_location(&location).with_tag(&REPORT_INFO_LINE_TEXT));
  }

  report.apply(callback);
}

impl<'pool, U: UserReadError> From<ByteOrderMarkError<'pool, U>> for BymlHeaderReadError<'pool, U> {
  fn from(value: ByteOrderMarkError<'pool, U>) -> Self {
    Self::ByteOrderMark(value)
  }
}
