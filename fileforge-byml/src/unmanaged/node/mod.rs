pub mod binary;
pub mod dictionary;
pub mod string;
pub mod r#trait;

use std::fmt::Debug;

use r#trait::BymlNodeReader;

use fileforge_lib::{
  diagnostic::node::{name::DiagnosticNodeName, reference::DiagnosticReference},
  error::Error,
  provider::{
    error::{slice_error::SliceError, ProviderError},
    out_of_bounds::SliceOutOfBoundsError,
    r#trait::Provider,
  },
  reader::{
    endianness::Endianness, error::underlying_provider_stat::UnderlyingProviderStatError, Reader,
  },
};

use super::BymlReader;

pub struct BymlReaderNode<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  BP: Provider,
> {
  pub(super) r#type: u8,
  pub(super) value: u32,
  pub(super) disable_inline: bool,
  pub(super) endianness: Endianness,
  pub(super) byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
}

#[derive(Debug)]
pub enum BymlDowncastError<T, Se: ProviderError> {
  TypeIdMismatch,
  RefOutOfBounds(SliceOutOfBoundsError),
  StatError(Se),
  TypeError(T),
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  BymlReaderNode<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  pub fn downcast<T: BymlNodeReader<'byml, 'byml, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>>(
    &self,
  ) -> Result<T, BymlDowncastError<T::ReadError, BP::StatError>> {
    if !T::type_id_supported(self.r#type) {
      return Err(BymlDowncastError::TypeIdMismatch);
    }

    if !T::requires_dereferencing(self.r#type) && !self.disable_inline {
      return Ok(T::from_value(self.r#type, self.value, self.byml));
    }

    let provider = self
      .byml
      .provider
      .slice_dyn(self.value as u64, None)
      .map_err(|e| match e {
        SliceError::OutOfBounds(oob) => BymlDowncastError::RefOutOfBounds(oob),
        SliceError::StatError(se) => BymlDowncastError::StatError(se.0),
      })?;

    let reader = Reader::at(
      provider,
      self.endianness,
      self
        .byml
        .diagnostic_root()
        .unwrap_or_else(|_| DiagnosticReference::new_invalid_from_pool(self.byml.pool))
        .create_physical_child(
          self.value as u64,
          None,
          DiagnosticNodeName::from("BymlNode"),
        ),
    );

    T::from_reader(self.r#type, reader, self.byml).map_err(|e| BymlDowncastError::TypeError(e))
  }
}
