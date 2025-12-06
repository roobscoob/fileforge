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

pub enum Yaz0HeaderReadError<'pool, U: UserReadError> {
  Magic(MagicError<'pool, 4, U>),
  TotalSize(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  Alignment(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  Unused(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
}

impl<'pool, U: UserReadError> FileforgeError for Yaz0HeaderReadError<'pool, U> {
  fn render_into_report<P: fileforge::diagnostic::pool::DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}
