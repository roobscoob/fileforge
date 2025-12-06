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

use crate::sead::sarc::header::SarcHeader;

pub const SARC_MAGIC: Magic<4> = Magic::from_byte_ref(b"SARC");
pub const SARC_BOM: ByteOrderMark = ByteOrderMark::from_byte_ref(Endianness::BigEndian, &[0xFE, 0xFF]);

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for SarcHeader {
  type Error = SarcHeaderReadError<'pool, S::ReadError>;
  type Argument = ();

  async fn read(reader: &mut fileforge::binary_reader::BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    reader.read_with::<Magic<4>>(SARC_MAGIC).await.map_err(|e| SarcHeaderReadError::Magic(e))?;

    let _header_length: u16 = reader.get().await.map_err(|e| SarcHeaderReadError::HeaderLength(e))?;

    let endianness = reader.read_with::<ByteOrderMark>(SARC_BOM).await.map_err(|e| SarcHeaderReadError::BOM(e))?.endianness();

    let mut reader = reader.borrow_fork();

    reader.set_endianness(endianness);

    let size = reader.get().await.map_err(|e| SarcHeaderReadError::Size(e))?;
    let data_section_offset = reader.get().await.map_err(|e| SarcHeaderReadError::DataSectionOffset(e))?;
    let version: u16 = reader.get().await.map_err(|e| SarcHeaderReadError::Version(e))?;
    let version = ((version >> 8) as u8, (version & 0xFF) as u8);

    let _unused: u16 = reader.get().await.map_err(|e| SarcHeaderReadError::Unused(e))?;

    Ok(SarcHeader {
      endianness,
      size,
      version,
      data_section_offset,
    })
  }
}

pub enum SarcHeaderReadError<'pool, U: UserReadError> {
  Magic(MagicError<'pool, 4, U>),
  BOM(ByteOrderMarkError<'pool, U>),
  Size(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  DataSectionOffset(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  Version(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  Unused(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  HeaderLength(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
}

impl<'pool, U: UserReadError> FileforgeError for SarcHeaderReadError<'pool, U> {
  fn render_into_report<P: fileforge::diagnostic::pool::DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    todo!()
  }
}
