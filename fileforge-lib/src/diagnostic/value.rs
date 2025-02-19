use core::ops::Deref;

use super::node::reference::DiagnosticReference;

pub struct DiagnosticValue<'pool, T, const NODE_NAME_SIZE: usize>(pub T, pub Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>);

impl<'pool, T: Clone, const NODE_NAME_SIZE: usize> Clone for DiagnosticValue<'pool, T, NODE_NAME_SIZE> {
  fn clone(&self) -> Self {
    Self(self.0.clone(), self.1)
  }
}

impl<'pool, T: Copy, const NODE_NAME_SIZE: usize> Copy for DiagnosticValue<'pool, T, NODE_NAME_SIZE> {}

impl<'pool, T, const NODE_NAME_SIZE: usize> Deref for DiagnosticValue<'pool, T, NODE_NAME_SIZE> {
  type Target = T;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'pool, T, const NODE_NAME_SIZE: usize> DiagnosticValue<'pool, T, NODE_NAME_SIZE> {
  pub fn reference(&self) -> Option<DiagnosticReference<'pool, NODE_NAME_SIZE>> { self.1 }

  pub fn value_ref<'t>(&'t self) -> &'t T { &self.0 }
}
