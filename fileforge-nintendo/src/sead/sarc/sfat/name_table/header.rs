use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  diagnostic::value::{DiagnosticSaturation, DiagnosticValue},
  error::{ext::annotations::annotated::Annotated, FileforgeError},
  stream::{error::user_read::UserReadError, ReadableStream},
};
use fileforge_macros::story;
use fileforge_std::magic::{Magic, MagicError};

use crate::report::{render_field_read, render_magic, render_wrong_value, Field, MagicMeaning};

pub const SFNT_MAGIC: Magic<4> = Magic::from_byte_ref(b"SFNT");

/// The SFNT header's size. The names start right after it.
pub const SFNT_HEADER_SIZE: u16 = 0x8;

/// The header of the name table, which holds each file's name.
pub struct SfntHeader;

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for SfntHeader {
  type Error = SfntHeaderReadError<'pool, S::ReadError>;
  type Argument = ();

  async fn read(reader: &mut BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    reader.read_with::<Magic<4>>(SFNT_MAGIC).await.map_err(|e| SfntHeaderReadError::Magic(e))?;

    let header_length: u16 = reader.get().await.map_err(|e| SfntHeaderReadError::HeaderLength(e))?;

    if header_length != SFNT_HEADER_SIZE {
      return Err(SfntHeaderReadError::WrongHeaderLength(reader.create_physical_diagnostic(-2, Some(2), "HeaderLength").saturate(header_length)));
    }

    let _reserved: u16 = reader.get().await.map_err(|e| SfntHeaderReadError::Reserved(e))?;

    Ok(SfntHeader)
  }
}

#[story("invalid magic", SfntHeaderReadError::Magic({
  let error: fileforge_std::magic::MagicError<'_, 4, StoryUserError> = fileforge_std::magic::MagicError::Invalid {
    actual: dv!("archive.sarc" / "Magic" @ 48..52 [fileforge_std::magic::Magic::from_bytes(*b"BAD!")]),
    expected: fileforge_std::magic::Magic::from_bytes(*b"SFNT"),
  };
  error
}))]
#[story("file ends inside the header length", SfntHeaderReadError::HeaderLength(read_exhausted::<u16, StoryUserError>(dr!("archive.sarc" @ 0..53), 52, DiagnosticValue(53, None))))]
#[story("wrong header length", SfntHeaderReadError::<StoryUserError>::WrongHeaderLength(dv!("archive.sarc" / "HeaderLength" @ 52..54 [0x10u16])))]
#[story("file ends inside the reserved field", SfntHeaderReadError::Reserved(read_exhausted::<u16, StoryUserError>(dr!("archive.sarc" @ 0..55), 54, DiagnosticValue(55, None))))]
pub enum SfntHeaderReadError<'pool, U: UserReadError> {
  Magic(MagicError<'pool, 4, U>),
  HeaderLength(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  /// The header length isn't [`SFNT_HEADER_SIZE`].
  WrongHeaderLength(DiagnosticValue<'pool, u16>),
  Reserved(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
}

impl<'pool, U: UserReadError> FileforgeError for SfntHeaderReadError<'pool, U> {
  fn render_into_report<P: fileforge::diagnostic::pool::DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    let field = |name, kind| Field { structure: "SFNT header", name, kind };

    match self {
      Self::Magic(error) => render_magic::<Self, 4, U, P, ITEM_NAME_SIZE>(
        error,
        MagicMeaning { structure: "SFNT header", title: "Missing SFNT section", subject: "An SFNT section" },
        provider,
        callback,
      ),
      Self::HeaderLength(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("header length", "a u16"), provider, callback),
      Self::WrongHeaderLength(actual) => {
        render_wrong_value::<Self, u16, P, ITEM_NAME_SIZE>(actual, SFNT_HEADER_SIZE, field("length", "a u16"), "Invalid SFNT header", provider, callback)
      }
      Self::Reserved(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("reserved field", "a u16"), provider, callback),
    }
  }
}
