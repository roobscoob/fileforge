use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::{error::user_read::UserReadError, ReadableStream},
};
use fileforge_macros::FileforgeError;
use fileforge_std::magic::{Magic, MagicError};

use super::Yaz0Header;

pub const YAZ0_MAGIC: Magic<4> = Magic::from_byte_ref(b"Yaz0");

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for Yaz0Header {
  type Error = Yaz0HeaderReadError<'pool, S::ReadError>;
  type Argument = ();

  async fn read(reader: &mut BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    reader.read_with::<Magic<4>>(YAZ0_MAGIC).await.map_err(|e| Yaz0HeaderReadError::Magic(e))?;

    Ok(Yaz0Header {
      decompressed_size: reader.get().await.map_err(|e| Yaz0HeaderReadError::TotalSize(e))?,
      data_alignment: reader.get().await.map_err(|e| Yaz0HeaderReadError::Alignment(e))?,
      unused: reader.get().await.map_err(|e| Yaz0HeaderReadError::Unused(e))?,
    })
  }
}

#[derive(FileforgeError)]
#[report(&"Failed to read Yaz0 Header")]
pub enum Yaz0HeaderReadError<'pool, U: UserReadError> {
  Magic(#[from] MagicError<'pool, 4, U>),
  TotalSize(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  Alignment(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  Unused(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
}
