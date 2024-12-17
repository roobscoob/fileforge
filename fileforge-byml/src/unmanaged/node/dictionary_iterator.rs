use fileforge_lib::{
  provider::r#trait::Provider, reader::error::underlying_provider_read::UnderlyingProviderReadError,
};

use super::dictionary::{BymlDictionaryNodeElement, BymlDictionaryNodeReader};

pub struct BymlDictionaryNodeReaderIterator<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  BP: Provider,
> {
  pub(super) index: usize,
  pub(super) reader:
    BymlDictionaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider> Iterator
  for BymlDictionaryNodeReaderIterator<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  type Item = Result<
    BymlDictionaryNodeElement<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
    UnderlyingProviderReadError<'pool, BP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>,
  >;

  fn next(&mut self) -> Option<Self::Item> {
    let v = self.reader.get_index(self.index);
    self.index += 1;

    match v {
      Ok(Some(v)) => Some(Ok(v)),
      Ok(None) => None,
      Err(e) => Some(Err(e)),
    }
  }
}
