use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::ReadableStream,
};
use fileforge_macros::FileforgeError;

use crate::byml::node::BymlNodeDiscriminants;

impl BymlNodeDiscriminants {
  fn resolve(id: u8) -> Result<BymlNodeDiscriminants, u8> {
    Ok(match id {
      0xA0 => BymlNodeDiscriminants::String,
      0xA1 => BymlNodeDiscriminants::BinaryData,
      0xA2 => BymlNodeDiscriminants::BinaryDataWithParameter,
      0xC0 => BymlNodeDiscriminants::Array,
      0xC1 => BymlNodeDiscriminants::Dictionary,
      0xC2 => BymlNodeDiscriminants::StringTable,
      0xC3 => BymlNodeDiscriminants::BinaryDataTable,
      0xD0 => BymlNodeDiscriminants::Bool,
      0xD1 => BymlNodeDiscriminants::Integer32,
      0xD2 => BymlNodeDiscriminants::Float32,
      0xD3 => BymlNodeDiscriminants::UnsignedInteger32,
      0xD4 => BymlNodeDiscriminants::Integer64,
      0xD5 => BymlNodeDiscriminants::UnsignedInteger64,
      0xD6 => BymlNodeDiscriminants::Float64,
      0xFF => BymlNodeDiscriminants::Null,
      _ => return Err(id),
    })
  }

  fn min_version(&self) -> Option<u16> {
    Some(match self {
      Self::BinaryDataTable => return None,

      Self::String => 1,
      Self::Array => 1,
      Self::Dictionary => 1,
      Self::StringTable => 1,
      Self::Bool => 1,
      Self::Integer32 => 1,
      Self::Float32 => 1,
      Self::Null => 1,

      Self::UnsignedInteger32 => 2,

      Self::Integer64 => 3,
      Self::UnsignedInteger64 => 3,
      Self::Float64 => 3,

      Self::BinaryData => 4,

      Self::BinaryDataWithParameter => 5,
    })
  }

  fn filter_version(self, version: BymlNodeDiscriminantVersionConfig) -> Option<Self> {
    if version.feat_binary_data_table && matches!(self, Self::BinaryData | Self::BinaryDataTable) {
      return Some(self);
    }

    self.min_version().is_some_and(|v| v <= version.version_number).then_some(self)
  }
}

#[derive(Clone, Copy, Debug)]
pub struct BymlNodeDiscriminantVersionConfig {
  pub version_number: u16,
  pub feat_binary_data_table: bool,
}

#[derive(FileforgeError)]
pub enum BymlNodeDiscriminantsReadError<'pool, S: ReadableStream> {
  ReadValue(#[from] Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>),

  #[report(&"Unknown Discriminant :(")]
  UnknownDiscriminant(u8),

  #[report(&"Invalid Version :O")]
  InvalidVersion,
}

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for BymlNodeDiscriminants {
  type Error = BymlNodeDiscriminantsReadError<'pool, S>;
  type Argument = BymlNodeDiscriminantVersionConfig;

  async fn read(reader: &mut BinaryReader<'pool, S>, version: Self::Argument) -> Result<Self, Self::Error> {
    BymlNodeDiscriminants::resolve(reader.read().await?)
      .map_err(|e| BymlNodeDiscriminantsReadError::UnknownDiscriminant(e))?
      .filter_version(version)
      .ok_or(BymlNodeDiscriminantsReadError::InvalidVersion)
  }
}
