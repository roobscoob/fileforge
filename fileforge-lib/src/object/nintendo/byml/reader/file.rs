use crate::{provider::r#trait::Provider, reader::Reader};

pub struct BymlFile<'pool, 'provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  over: Reader<'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P>,
}

impl<'pool, 'provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> BymlFile<'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P> {
  pub (super) fn dereference_value_reference(&self, value: u32) {

  }
}