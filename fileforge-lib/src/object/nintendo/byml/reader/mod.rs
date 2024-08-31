use file::BymlFile;

use crate::{provider::r#trait::Provider, reader::Reader};

pub mod node;
pub mod file;

pub struct BymlReader<'file, 'pool, 'provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  kind: u8,
  over: Reader<'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P>,
  file: &'file BymlFile<'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P>,
}

impl<'file, 'pool, 'provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> BymlReader<'file, 'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P> {
  
}