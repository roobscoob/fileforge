use core::ops::Deref;

use crate::diagnostic::{node::reference::DislocatedDiagnosticReference, pool::DiagnosticPoolBuilder};

use super::node::reference::DiagnosticReference;

pub struct DiagnosticValue<'pool, T>(pub T, pub Option<DiagnosticReference<'pool>>);
pub struct DislocatedDiagnosticValue<T>(pub T, pub Option<DislocatedDiagnosticReference>);

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

  pub fn value<'t>(&'t self) -> &'t T {
    &self.0
  }

  pub fn dislocate(self) -> DislocatedDiagnosticValue<T> {
    DislocatedDiagnosticValue(self.0, self.1.map(|v| v.dislocate()))
  }

  pub fn map<N>(self, transformer: impl FnOnce(T) -> N) -> DiagnosticValue<'pool, N> {
    DiagnosticValue(transformer(self.0), self.1)
  }
}

impl<T> DislocatedDiagnosticValue<T> {
  pub fn reference(&self) -> Option<DislocatedDiagnosticReference> {
    self.1
  }

  pub fn value_ref<'t>(&'t self) -> &'t T {
    &self.0
  }

  pub fn relocate<'pool>(self, pool: &'pool dyn DiagnosticPoolBuilder) -> DiagnosticValue<'pool, T> {
    DiagnosticValue(self.0, self.1.map(|v| v.relocate(pool)))
  }

  pub fn map<N>(self, transformer: impl FnOnce(T) -> N) -> DislocatedDiagnosticValue<N> {
    DislocatedDiagnosticValue(transformer(self.0), self.1)
  }
}

pub trait DiagnosticSaturation<'pool, T> {
  fn saturate(self, with: T) -> DiagnosticValue<'pool, T>;
}

impl<'pool, T> DiagnosticSaturation<'pool, T> for Option<DiagnosticReference<'pool>> {
  fn saturate(self, with: T) -> DiagnosticValue<'pool, T> {
    DiagnosticValue(with, self)
  }
}
