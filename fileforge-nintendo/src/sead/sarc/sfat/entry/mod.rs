use fileforge_macros::story;
pub mod attributes;

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

use crate::sead::sarc::sfat::entry::attributes::{FilenameAttributes, FilenameAttributesError};
use fileforge::error::render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::r#const::ConstText};
use crate::report::{render_field_read, render_with_context, Field};

pub const SFAT_ENTRY_SIZE: u64 = 0x10;

pub struct SfatEntry {
  pub filename_hash: u32,
  pub filename_attributes: Option<FilenameAttributes>,
  pub start_offset: u32,
  pub end_offset: u32,
}

#[story("file ends inside the filename hash", SfatEntryError::FilenameHashReadError(read_exhausted::<u32, StoryUserError>(dr!("archive.sarc" @ 0..34), 32, DiagnosticValue(34, None))))]
#[story("file ends inside the filename attributes", SfatEntryError::FilenameAttributesReadError(read_exhausted::<u32, StoryUserError>(dr!("archive.sarc" @ 0..38), 36, DiagnosticValue(38, None))))]
#[story("file ends inside the start offset", SfatEntryError::StartOffsetReadError(read_exhausted::<u32, StoryUserError>(dr!("archive.sarc" @ 0..42), 40, DiagnosticValue(42, None))))]
#[story("file ends inside the end offset", SfatEntryError::EndOffsetReadError(read_exhausted::<u32, StoryUserError>(dr!("archive.sarc" @ 0..46), 44, DiagnosticValue(46, None))))]
#[story("invalid filename attributes", SfatEntryError::<StoryUserError>::FilenameAttributesError(FilenameAttributesError::ZeroSequence))]
pub enum SfatEntryError<'pool, U: UserReadError> {
  FilenameHashReadError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  StartOffsetReadError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  EndOffsetReadError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  FilenameAttributesReadError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  FilenameAttributesError(FilenameAttributesError),
}

const READING_SFAT_ENTRY: ConstText = ConstText::new("This happened while reading an SFAT entry.", &REPORT_INFO_LINE_TEXT);

impl<'pool, U: UserReadError> FileforgeError for SfatEntryError<'pool, U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    let field = |name, kind| Field { structure: "SFAT entry", name, kind };

    match self {
      Self::FilenameHashReadError(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("filename hash", "a u32"), provider, callback),
      Self::FilenameAttributesReadError(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("filename attributes", "a u32"), provider, callback),
      Self::StartOffsetReadError(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("data start offset", "a u32"), provider, callback),
      Self::EndOffsetReadError(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("data end offset", "a u32"), provider, callback),
      Self::FilenameAttributesError(error) => render_with_context(error, &READING_SFAT_ENTRY, provider, callback),
    }
  }
}

impl<'pool, U: UserReadError> From<FilenameAttributesError> for SfatEntryError<'pool, U> {
  fn from(value: FilenameAttributesError) -> Self {
    Self::FilenameAttributesError(value)
  }
}

impl<'pool, U: UserReadError> UserReadError for SfatEntryError<'pool, U> {}

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for SfatEntry {
  type Argument = ();
  type Error = SfatEntryError<'pool, S::ReadError>;

  async fn read(reader: &mut BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    Ok(SfatEntry {
      filename_hash: reader.get().await.map_err(|e| SfatEntryError::FilenameHashReadError(e))?,
      filename_attributes: FilenameAttributes::from_bits(reader.get().await.map_err(|e| SfatEntryError::FilenameHashReadError(e))?)?,
      start_offset: reader.get().await.map_err(|e| SfatEntryError::StartOffsetReadError(e))?,
      end_offset: reader.get().await.map_err(|e| SfatEntryError::EndOffsetReadError(e))?,
    })
  }
}
