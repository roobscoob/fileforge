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
use fileforge_macros::FileforgeError;

use crate::sead::sarc::sfat::entry::attributes::{FilenameAttributes, FilenameAttributesError};

pub const SFAT_ENTRY_SIZE: u64 = 0x10;

pub struct SfatEntry {
  pub filename_hash: u32,
  pub filename_attributes: Option<FilenameAttributes>,
  pub start_offset: u32,
  pub end_offset: u32,
}

#[derive(FileforgeError)]
pub enum SfatEntryError<'pool, U: UserReadError> {
  FilenameHashReadError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  StartOffsetReadError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  EndOffsetReadError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  FilenameAttributesReadError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  FilenameAttributesError(#[from] FilenameAttributesError),
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
