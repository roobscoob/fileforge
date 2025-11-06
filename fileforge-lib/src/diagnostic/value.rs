use core::ops::Deref;

use super::node::reference::DiagnosticReference;

pub struct DiagnosticValue<'pool, T>(pub T, pub Option<DiagnosticReference<'pool>>);

impl<'pool, T: Clone> Clone for DiagnosticValue<'pool, T> {
  fn clone(&self) -> Self {
    Self(self.0.clone(), self.1)
  }
}

impl<'pool, T> From<T> for DiagnosticValue<'pool, T> {
  fn from(value: T) -> Self {
    Self(value, None)
  }
}

impl<'pool, T: Copy> Copy for DiagnosticValue<'pool, T> {}

impl<'pool, T> Deref for DiagnosticValue<'pool, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'pool, T> DiagnosticValue<'pool, T> {
  pub fn reference(&self) -> Option<DiagnosticReference<'pool>> {
    self.1
  }

  pub fn value_ref<'t>(&'t self) -> &'t T {
    &self.0
  }
}
