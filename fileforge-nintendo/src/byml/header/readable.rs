use fileforge::{
  binary_reader::{
    endianness::Endianness,
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::{error::user_read::UserReadError, ReadableStream},
};
use fileforge_macros::FileforgeError;
use fileforge_std::byte_order_mark::{error::ByteOrderMarkError, ByteOrderMark};

use crate::byml::header::BymlHeaderConfig;

use super::BymlHeader;

pub const BYML_BOM: ByteOrderMark = ByteOrderMark::from_byte_ref(Endianness::BigEndian, b"BY");

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for BymlHeader {
  type Error = BymlHeaderReadError<'pool, S::ReadError>;
  type Argument = BymlHeaderConfig;

  async fn read(reader: &mut BinaryReader<'pool, S>, config: Self::Argument) -> Result<Self, Self::Error> {
    reader.read_with::<ByteOrderMark>(BYML_BOM).await.map_err(|e| BymlHeaderReadError::ByteOrderMark(e))?;

    Ok(BymlHeader {
      config,
      version: reader.get().await.map_err(|e| BymlHeaderReadError::Version(e))?,
      key_table_offset: reader.get().await.map_err(|e| BymlHeaderReadError::KeyTableOffset(e))?,
      string_table_offset: reader.get().await.map_err(|e| BymlHeaderReadError::StringTableOffset(e))?,
      binary_data_table_offset: if config.include_binary_data_table {
        reader.get().await.map_err(|e| BymlHeaderReadError::BinaryDataTableOffset(e))?
      } else {
        0
      },
      root_data_offset: reader.get().await.map_err(|e| BymlHeaderReadError::RootDataOffset(e))?,
    })
  }
}

#[derive(FileforgeError)]
#[report(&"Failed to read Byml Header")]
pub enum BymlHeaderReadError<'pool, U: UserReadError> {
  ByteOrderMark(#[from] ByteOrderMarkError<'pool, U>),
  Version(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  KeyTableOffset(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  StringTableOffset(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  BinaryDataTableOffset(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  RootDataOffset(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
}
