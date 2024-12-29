use core::ops::Deref;

use super::node::reference::DiagnosticReference;

pub struct DiagnosticValue<'pool, T, const NODE_NAME_SIZE: usize>(T, DiagnosticReference<'pool, NODE_NAME_SIZE>);

impl<'pool, T, const NODE_NAME_SIZE: usize> Deref for DiagnosticValue<'pool, T, NODE_NAME_SIZE> {
  type Target = T;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'pool, T, const NODE_NAME_SIZE: usize> DiagnosticValue<'pool, T, NODE_NAME_SIZE> {
  pub fn reference(&self) -> DiagnosticReference<'pool, NODE_NAME_SIZE> { self.1 }

  pub fn value_ref<'t>(&'t self) -> &'t T { &self.0 }
}
