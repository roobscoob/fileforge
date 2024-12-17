pub mod dictionary;
pub mod dictionary_iterator;
pub mod error;
pub mod r#trait;

use r#trait::BymlNodeReader;

use fileforge_lib::{
  diagnostic::node::name::DiagnosticNodeName,
  provider::{out_of_bounds::SliceOutOfBoundsError, r#trait::Provider},
  reader::{endianness::Endianness, Reader},
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
pub enum BymlDowncastError {
  TypeIdMismatch,
  RefOutOfBounds(SliceOutOfBoundsError),
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  BymlReaderNode<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  pub fn downcast<T: BymlNodeReader<'byml, 'byml, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>>(
    &self,
  ) -> Result<T, BymlDowncastError> {
    if !T::type_id_supported(self.r#type) {
      return Err(BymlDowncastError::TypeIdMismatch);
    }

    if !T::requires_dereferencing(self.r#type) && !self.disable_inline {
      return Ok(T::from_value(self.r#type, self.value, self.byml));
    }

    let len = self.byml.provider.len().saturating_sub(self.value as u64);
    let provider = self
      .byml
      .provider
      .slice_dyn(self.value as u64, len)
      .map_err(|e| BymlDowncastError::RefOutOfBounds(e))?;

    let reader = Reader::at(
      provider,
      self.endianness,
      self.byml.diagnostic_root().create_physical_child(
        self.value as u64,
        len,
        DiagnosticNodeName::from("BymlNode"),
      ),
    );

    Ok(T::from_reader(self.r#type, reader, self.byml))
  }
}
