pub mod error;

use std::ffi::CStr;

use error::string_node_get_content::StringNodeGetContent;
use fileforge_lib::{
  diagnostic::node::name::DiagnosticNodeName,
  provider::{error::never::Never, r#trait::Provider},
  reader::error::{
    parse::ParseError, parse_primitive::ParsePrimitiveError,
    underlying_provider_stat::UnderlyingProviderStatError,
  },
};

use crate::unmanaged::BymlReader;

use super::r#trait::BymlNodeReader;

pub struct BymlStringNodeReader<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  BP: Provider,
> {
  index: u32,
  byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  BymlStringNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  pub fn with_content<T>(
    &self,
    mapper: impl for<'t> FnOnce(&'t CStr) -> T,
  ) -> Result<T, StringNodeGetContent<'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>> {
    let st = self
      .byml
      .string_table()
      .map_err(|e| StringNodeGetContent::GetStringTableError(e))?;

    if let Some(mut st) = st {
      st.try_get(self.index, mapper)
        .map_err(|e| StringNodeGetContent::GetStringTableEntryError(e))
    } else {
      Err(StringNodeGetContent::NoStringTable)
    }
  }
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  BymlNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
  for BymlStringNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  type ReadError =
    ParsePrimitiveError<'pool, BP::ReadError, BP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>;

  fn requires_dereferencing(type_id: u8) -> bool { false }
  fn type_id_supported(type_id: u8) -> bool { type_id == 0xA0 }

  fn from_value(
    type_id: u8,
    value: u32,
    byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Self {
    Self {
      index: value,
      byml: byml,
    }
  }

  fn from_reader(
    type_id: u8,
    reader: fileforge_lib::reader::Reader<
      'pool,
      DIAGNOSTIC_NODE_NAME_SIZE,
      <BP as Provider>::DynReturnedProviderType<'byml_provider>,
    >,
    byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Result<Self, Self::ReadError> {
    let index = reader.get::<4, u32>("Index").map_err(|e| match e {
      ParsePrimitiveError::OutOfBounds(oob) => ParsePrimitiveError::OutOfBounds(oob),
      ParsePrimitiveError::UnderlyingProviderReadError(re) => {
        ParsePrimitiveError::UnderlyingProviderReadError(re)
      }
      ParsePrimitiveError::UnderlyingProviderStatError(UnderlyingProviderStatError(se)) => {
        ParsePrimitiveError::UnderlyingProviderStatError(UnderlyingProviderStatError(se))
      }
    })?;

    Ok(Self::from_value(type_id, index, byml))
  }
}
