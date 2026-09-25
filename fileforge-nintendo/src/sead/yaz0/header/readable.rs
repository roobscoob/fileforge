use fileforge_macros::story;
use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::{error::user_read::UserReadError, ReadableStream},
};
use fileforge::{diagnostic::pool::DiagnosticPoolProvider, error::{report::Report, FileforgeError}};
use fileforge_std::magic::{Magic, MagicError};

use super::Yaz0Header;
use crate::report::{render_field_read, render_magic, Field, MagicMeaning};

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

// TODO: the derive silently ignored this enum-level title: "Failed to read Yaz0 Header".
#[story("invalid magic", Yaz0HeaderReadError::Magic({
  let error: fileforge_std::magic::MagicError<'_, 4, StoryUserError> = fileforge_std::magic::MagicError::Invalid {
    actual: dv!("archive.szs" / "Magic" @ 0..4 [fileforge_std::magic::Magic::from_bytes(*b"BAD!")]),
    expected: fileforge_std::magic::Magic::from_bytes(*b"Yaz0"),
  };
  error
}))]
#[story("file ends inside the decompressed size", Yaz0HeaderReadError::TotalSize(read_exhausted::<u32, StoryUserError>(dr!("archive.szs" @ 0..6), 4, DiagnosticValue(6, None))))]
#[story("file ends inside the alignment", Yaz0HeaderReadError::Alignment(read_exhausted::<u32, StoryUserError>(dr!("archive.szs" @ 0..10), 8, DiagnosticValue(10, None))))]
#[story("file ends inside the reserved field", Yaz0HeaderReadError::Unused(read_exhausted::<u32, StoryUserError>(dr!("archive.szs" @ 0..14), 12, DiagnosticValue(14, None))))]
pub enum Yaz0HeaderReadError<'pool, U: UserReadError> {
  Magic(MagicError<'pool, 4, U>),
  TotalSize(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  Alignment(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  Unused(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
}

impl<'pool, U: UserReadError> FileforgeError for Yaz0HeaderReadError<'pool, U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    let field = |name, kind| Field { structure: "Yaz0 header", name, kind };

    match self {
      Self::TotalSize(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("decompressed size", "a u32"), provider, callback),
      Self::Alignment(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("alignment", "a u32"), provider, callback),
      Self::Unused(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("reserved field", "a u32"), provider, callback),
      Self::Magic(error) => render_magic::<Self, 4, U, P, ITEM_NAME_SIZE>(
        error,
        MagicMeaning { structure: "Yaz0 header", title: "Not Yaz0-compressed data", subject: "Yaz0-compressed data" },
        provider,
        callback,
      ),
    }
  }
}

impl<'pool, U: UserReadError> From<MagicError<'pool, 4, U>> for Yaz0HeaderReadError<'pool, U> {
  fn from(value: MagicError<'pool, 4, U>) -> Self {
    Self::Magic(value)
  }
}
