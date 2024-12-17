pub mod dictionary;
pub mod error;
pub mod r#trait;

use r#trait::BymlNodeReader;

use fileforge_lib::{provider::r#trait::Provider, reader::Reader};

use super::BymlReader;

pub struct BymlReaderNode<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  RP: Provider,
  BP: Provider,
> {
  pub(super) r#type: u8,
  pub(super) reader: Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>,
  pub(super) byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
}

impl<
    'byml,
    'byml_provider,
    'pool,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
    RP: Provider,
    BP: Provider,
  > BymlReaderNode<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP, BP>
{
  fn downcast_ref<T: BymlNodeReader<'byml, 'byml, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP, BP>>(
    &self,
  ) -> Option<T>
  where
    RP: Clone,
  {
    if !T::type_id_supported(self.r#type) {
      return None;
    }

    return Some(T::from_reader(self.r#type, self.reader.clone(), self.byml));
  }

  fn downcast<T: BymlNodeReader<'byml, 'byml, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP, BP>>(
    self,
  ) -> Option<T> {
    if T::type_id_supported(self.r#type) {
      Some(T::from_reader(self.r#type, self.reader, self.byml))
    } else {
      None
    }
  }
}
