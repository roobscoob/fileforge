use crate::{object::nintendo::byml::reader::{file::BymlFile, BymlReader}, provider::r#trait::Provider};

use super::BymlNodeReader;

pub struct BymlArrayReader<'file, 'pool, 'provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  reader: BymlReader<'file, 'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P>,
  
  // max u24::Max
  length: u32,
}

impl<'file, 'pool, 'provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider>
  BymlNodeReader<'file, 'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P>
  for BymlArrayReader<'file, 'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P> {

  fn from_value(value: u32, file: &BymlFile<'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P>) -> Self {
    file.dereference_value_reference(value)
  }
}