use fileforge::{
  binary_reader::{
    error::{primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  error::{ext::annotations::annotated::Annotated, FileforgeError},
  stream::{error::user_read::UserReadError, ReadableStream},
};
use fileforge_std::magic::{Magic, MagicError};

use super::SfatHeader;

pub const SFAT_MAGIC: Magic<4> = Magic::from_byte_ref(b"SFAT");

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for SfatHeader {
  type Error = SfatHeaderReadError<'pool, S::ReadError>;
  type Argument = ();

  async fn read(reader: &mut BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    reader.read_with::<Magic<4>>(SFAT_MAGIC).await.map_err(|e| SfatHeaderReadError::Magic(e))?;

    let _header_length: u16 = reader.get().await.map_err(|e| SfatHeaderReadError::HeaderLength(e))?;

    Ok(SfatHeader {
      file_count: reader.get().await.map_err(|e| SfatHeaderReadError::FileCount(e))?,
      hash_multiplier: reader.get().await.map_err(|e| SfatHeaderReadError::HashMultiplier(e))?,
    })
  }
}

pub enum SfatHeaderReadError<'pool, U: UserReadError> {
  Magic(MagicError<'pool, 4, U>),
  HeaderLength(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  FileCount(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  HashMultiplier(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
}

impl<'pool, U: UserReadError> FileforgeError for SfatHeaderReadError<'pool, U> {
  fn render_into_report<P: fileforge::diagnostic::pool::DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}
