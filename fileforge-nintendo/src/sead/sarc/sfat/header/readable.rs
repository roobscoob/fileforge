use fileforge_macros::story;
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
use crate::report::{render_field_read, render_magic, Field, MagicMeaning};

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

#[story("invalid magic", SfatHeaderReadError::Magic({
  let error: fileforge_std::magic::MagicError<'_, 4, StoryUserError> = fileforge_std::magic::MagicError::Invalid {
    actual: dv!("archive.sarc" / "Magic" @ 0..4 [fileforge_std::magic::Magic::from_bytes(*b"BAD!")]),
    expected: fileforge_std::magic::Magic::from_bytes(*b"SFAT"),
  };
  error
}))]
#[story("file ends inside the header length", SfatHeaderReadError::HeaderLength(read_exhausted::<u16, StoryUserError>(dr!("archive.sarc" @ 0..25), 24, DiagnosticValue(25, None))))]
#[story("file ends inside the file count", SfatHeaderReadError::FileCount(read_exhausted::<u16, StoryUserError>(dr!("archive.sarc" @ 0..27), 26, DiagnosticValue(27, None))))]
#[story("file ends inside the hash multiplier", SfatHeaderReadError::HashMultiplier(read_exhausted::<u32, StoryUserError>(dr!("archive.sarc" @ 0..30), 28, DiagnosticValue(30, None))))]
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
    let field = |name, kind| Field { structure: "SFAT header", name, kind };

    match self {
      Self::HeaderLength(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("header length", "a u16"), provider, callback),
      Self::FileCount(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("file count", "a u16"), provider, callback),
      Self::HashMultiplier(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("hash multiplier", "a u32"), provider, callback),
      Self::Magic(error) => render_magic::<Self, 4, U, P, ITEM_NAME_SIZE>(
        error,
        MagicMeaning { structure: "SFAT header", title: "Missing SFAT section", subject: "An SFAT section" },
        provider,
        callback,
      ),
    }
  }
}
