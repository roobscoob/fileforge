use fileforge_macros::story;
use core::ops::Sub;

use fileforge::{
  binary_reader::{
    endianness::Endianness,
    error::{primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    PrimitiveReader,
  },
  error::{ext::annotations::annotated::Annotated, FileforgeError},
  stream::{error::user_read::UserReadError, ReadableStream},
};

use fileforge_std::{
  byte_order_mark::{error::ByteOrderMarkError, ByteOrderMark},
  magic::{Magic, MagicError},
};

use crate::sead::sarc::{header::SarcHeader, sfat::SfatTable};
use fileforge::error::render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::r#const::ConstText};
use crate::report::{render_field_read, render_magic, render_with_context, Field, MagicMeaning};

pub const SARC_MAGIC: Magic<4> = Magic::from_byte_ref(b"SARC");
pub const SARC_BOM: ByteOrderMark = ByteOrderMark::from_byte_ref(Endianness::BigEndian, &[0xFE, 0xFF]);

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for SarcHeader<'pool, S> {
  type Error = SarcHeaderReadError<'pool, S::ReadError>;
  type Argument = ();

  async fn read(reader: &mut fileforge::binary_reader::BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    reader.read_with::<Magic<4>>(SARC_MAGIC).await.map_err(|e| SarcHeaderReadError::Magic(e))?;

    let header_length: u16 = reader.get().await.map_err(|e| SarcHeaderReadError::HeaderLength(e))?;

    if header_length != 0x14 {
      panic!("todo: make this an error");
    }

    let endianness = reader.read_with::<ByteOrderMark>(SARC_BOM).await.map_err(|e| SarcHeaderReadError::BOM(e))?.endianness();

    let mut reader = reader.borrow_fork();

    reader.set_endianness(endianness);

    let size: u32 = reader.get().await.map_err(|e| SarcHeaderReadError::Size(e))?;
    let data_section_offset: u32 = reader.get().await.map_err(|e| SarcHeaderReadError::DataSectionOffset(e))?;
    let version: u16 = reader.get().await.map_err(|e| SarcHeaderReadError::Version(e))?;
    let version = ((version >> 8) as u8, (version & 0xFF) as u8);

    let _unused: u16 = reader.get().await.map_err(|e| SarcHeaderReadError::Unused(e))?;

    let Some(sfat_table_size) = data_section_offset.checked_sub(header_length as u32) else {
      panic!("todo: make this an error");
    };

    todo!();
    // Ok(SarcHeader {
    //   endianness,
    //   size,
    //   version,
    //   data_section_offset,
    //   sfat_table:
    // })
  }
}

#[story("file ends inside the magic", SarcHeaderReadError::Magic(fileforge_std::magic::MagicError::Failed(read_exhausted::<[u8; 4], StoryUserError>(dr!("archive.sarc" @ 0..2), 0, DiagnosticValue(2, None)))))]
#[story("file ends inside the byte order mark", SarcHeaderReadError::BOM(fileforge_std::byte_order_mark::error::ByteOrderMarkError::Failed(read_exhausted::<[u8; 2], StoryUserError>(dr!("archive.sarc" @ 0..7), 6, DiagnosticValue(7, None)))))]
#[story("invalid magic", SarcHeaderReadError::Magic({
  let error: fileforge_std::magic::MagicError<'_, 4, StoryUserError> = fileforge_std::magic::MagicError::Invalid {
    actual: dv!("archive.sarc" / "Magic" @ 0..4 [fileforge_std::magic::Magic::from_bytes(*b"BAD!")]),
    expected: fileforge_std::magic::Magic::from_bytes(*b"SARC"),
  };
  error
}))]
#[story("invalid byte order mark", SarcHeaderReadError::<StoryUserError>::BOM(fileforge_std::byte_order_mark::error::ByteOrderMarkError::Invalid(fileforge_std::byte_order_mark::error::invalid::ByteOrderMarkInvalid {
  expected: fileforge_std::byte_order_mark::ByteOrderMark::from_bytes(fileforge::binary_reader::endianness::Endianness::BigEndian, [0xFE, 0xFF]),
  actual: dv!("archive.sarc" / "ByteOrderMark" @ 6..8 [[0x12, 0x34]]),
})))]
#[story("file ends inside the header length", SarcHeaderReadError::HeaderLength(read_exhausted::<u16, StoryUserError>(dr!("archive.sarc" @ 0..5), 4, DiagnosticValue(5, None))))]
#[story("file ends inside the file size", SarcHeaderReadError::Size(read_exhausted::<u32, StoryUserError>(dr!("archive.sarc" @ 0..10), 8, DiagnosticValue(10, None))))]
#[story("file ends inside the data section offset", SarcHeaderReadError::DataSectionOffset(read_exhausted::<u32, StoryUserError>(dr!("archive.sarc" @ 0..14), 12, DiagnosticValue(14, None))))]
#[story("file ends inside the version", SarcHeaderReadError::Version(read_exhausted::<u16, StoryUserError>(dr!("archive.sarc" @ 0..17), 16, DiagnosticValue(17, None))))]
#[story("file ends inside the reserved field", SarcHeaderReadError::Unused(read_exhausted::<u16, StoryUserError>(dr!("archive.sarc" @ 0..19), 18, DiagnosticValue(19, None))))]
pub enum SarcHeaderReadError<'pool, U: UserReadError> {
  Magic(MagicError<'pool, 4, U>),
  BOM(ByteOrderMarkError<'pool, U>),
  Size(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  DataSectionOffset(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  Version(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  Unused(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  HeaderLength(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
}

const SARC_BYTE_ORDER_MARK: ConstText = ConstText::new("This is the SARC header's byte order mark.", &REPORT_INFO_LINE_TEXT);

impl<'pool, U: UserReadError> FileforgeError for SarcHeaderReadError<'pool, U> {
  fn render_into_report<P: fileforge::diagnostic::pool::DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    let field = |name, kind| Field { structure: "SARC header", name, kind };

    match self {
      Self::HeaderLength(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("header length", "a u16"), provider, callback),
      Self::Size(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("file size", "a u32"), provider, callback),
      Self::DataSectionOffset(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("data offset", "a u32"), provider, callback),
      Self::Version(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("version", "a u16"), provider, callback),
      Self::Unused(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("reserved field", "a u16"), provider, callback),
      Self::Magic(error) => render_magic::<Self, 4, U, P, ITEM_NAME_SIZE>(
        error,
        MagicMeaning { structure: "SARC header", title: "Not a SARC archive", subject: "A SARC archive" },
        provider,
        callback,
      ),
      Self::BOM(ByteOrderMarkError::Failed(read)) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("byte order mark", "2 bytes"), provider, callback),
      Self::BOM(ByteOrderMarkError::Invalid(invalid)) => render_with_context(invalid, &SARC_BYTE_ORDER_MARK, provider, callback),
    }
  }
}
