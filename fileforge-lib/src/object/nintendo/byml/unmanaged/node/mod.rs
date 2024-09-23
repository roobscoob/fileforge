use crate::provider::r#trait::Provider;

use super::BymlReader;

pub struct BymlReaderNode<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  reader: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>,
}