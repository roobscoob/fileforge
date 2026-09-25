use fileforge_macros::story;
use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::ReadableStream,
};
use fileforge::{diagnostic::pool::DiagnosticPoolProvider, error::{report::Report, FileforgeError}};

use crate::byml::node::{node_type_name, BymlNodeDiscriminants};
use fileforge::error::render::{buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT}, builtin::text::r#const::ConstText};
use fileforge::error::render::builtin::number::formatted_unsigned::FormattedUnsigned;
use fileforge_macros::text;
use crate::report::{render_field_read, Field, CORRUPTED};

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

#[story("file ends at the node type", BymlNodeDiscriminantsReadError::<StoryStream>::ReadValue(read_exhausted::<u8, StoryUserError>(dr!("data.byml" @ 0..32), 32, DiagnosticValue(32, None))))]
#[story("unknown node type", BymlNodeDiscriminantsReadError::<StoryStream>::UnknownDiscriminant(0x42))]
#[story("node type not in this version", {
  let error: BymlNodeDiscriminantsReadError<'_, StoryStream> = BymlNodeDiscriminantsReadError::InvalidVersion {
    node_type: BymlNodeDiscriminants::UnsignedInteger32,
    id: 0xD3,
    version: BymlNodeDiscriminantVersionConfig { version_number: 1, feat_binary_data_table: false },
  };
  error
})]
#[story("binary data table without the feature", {
  let error: BymlNodeDiscriminantsReadError<'_, StoryStream> = BymlNodeDiscriminantsReadError::InvalidVersion {
    node_type: BymlNodeDiscriminants::BinaryDataTable,
    id: 0xC3,
    version: BymlNodeDiscriminantVersionConfig { version_number: 7, feat_binary_data_table: false },
  };
  error
})]
pub enum BymlNodeDiscriminantsReadError<'pool, S: ReadableStream> {
  ReadValue(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>),
  UnknownDiscriminant(u8),
  /// `node_type` (read as `id`) isn't allowed by the file's `version`.
  InvalidVersion {
    node_type: BymlNodeDiscriminants,
    id: u8,
    version: BymlNodeDiscriminantVersionConfig,
  },
}

const NO_BINARY_DATA_TABLE: ConstText = ConstText::new("This file doesn't have one.", &REPORT_INFO_LINE_TEXT);
const UNKNOWN_NODE_TYPE: ConstText = ConstText::new("This usually means the file is corrupted, or uses a BYML version fileforge doesn't know.", &REPORT_FLAG_LINE_TEXT);

impl<'pool, S: ReadableStream> FileforgeError for BymlNodeDiscriminantsReadError<'pool, S> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::ReadValue(read) => render_field_read::<Self, S::ReadError, P, ITEM_NAME_SIZE>(
        read,
        Field { structure: "BYML node", name: "node type", kind: "a u8" },
        provider,
        callback,
      ),
      Self::UnknownDiscriminant(node_type) => {
        let node_type = FormattedUnsigned::new(*node_type as u128).base(16).uppercase().padding(2).prefix("0x");
        let found_text = text!([&REPORT_ERROR_TEXT] "Found node type {&node_type}, which isn't a BYML node type.");

        Report::new::<Self>(provider, &"Unknown BYML node type")
          .with_info_line(&found_text)
          .with_flag_line(&UNKNOWN_NODE_TYPE)
          .apply(callback)
      }
      Self::InvalidVersion { node_type, id, version } => {
        let id = FormattedUnsigned::new(*id as u128).base(16).uppercase().padding(2).prefix("0x");
        let name = node_type_name(*node_type);
        let file_version = FormattedUnsigned::new(version.version_number as u128);

        match node_type.min_version() {
          Some(min_version) => {
            let min_version = FormattedUnsigned::new(min_version as u128);
            let found_text = text!(
              { matches!(node_type, BymlNodeDiscriminants::BinaryData) }
                [&REPORT_ERROR_TEXT] "Found node type {&id} ({&name}), which BYML only allows from version {&min_version} on, or in files that have a binary data table.",

              [&REPORT_ERROR_TEXT] "Found node type {&id} ({&name}), which BYML only allows from version {&min_version} on."
            );
            let version_text = text!([&REPORT_INFO_LINE_TEXT] "This file is version {&file_version}.");

            Report::new::<Self>(provider, &"BYML node type not allowed here")
              .with_info_line(&found_text)
              .with_info_line(&version_text)
              .with_flag_line(&CORRUPTED)
              .apply(callback)
          }
          None => {
            let found_text = text!([&REPORT_ERROR_TEXT] "Found node type {&id} ({&name}), which is only allowed in files that have a binary data table.");

            Report::new::<Self>(provider, &"BYML node type not allowed here")
              .with_info_line(&found_text)
              .with_info_line(&NO_BINARY_DATA_TABLE)
              .with_flag_line(&CORRUPTED)
              .apply(callback)
          }
        }
      }
    }
  }
}

impl<'pool, S: ReadableStream> From<Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>> for BymlNodeDiscriminantsReadError<'pool, S> {
  fn from(value: Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>) -> Self {
    Self::ReadValue(value)
  }
}

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for BymlNodeDiscriminants {
  type Error = BymlNodeDiscriminantsReadError<'pool, S>;
  type Argument = BymlNodeDiscriminantVersionConfig;

  async fn read(reader: &mut BinaryReader<'pool, S>, version: Self::Argument) -> Result<Self, Self::Error> {
    let id: u8 = reader.read().await?;
    let node_type = BymlNodeDiscriminants::resolve(id).map_err(BymlNodeDiscriminantsReadError::UnknownDiscriminant)?;

    node_type.filter_version(version).ok_or(BymlNodeDiscriminantsReadError::InvalidVersion { node_type, id, version })
  }
}
