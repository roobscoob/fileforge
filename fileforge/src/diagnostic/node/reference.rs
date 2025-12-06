use core::{fmt::Debug, num::NonZero, u64};

use crate::diagnostic::pool::{DiagnosticPoolBuilder, DiagnosticPoolProvider};

use super::{branch::DiagnosticBranch, DiagnosticNode};

#[derive(Clone, Copy)]
pub struct DiagnosticReference<'pool> {
  pub(crate) index: u32,
  pub(crate) generation: NonZero<u32>,
  pub(crate) pool: &'pool dyn DiagnosticPoolBuilder,
}

impl<'pool> DiagnosticReference<'pool> {
  pub fn new_invalid(&self) -> DiagnosticReference<'pool> {
    DiagnosticReference {
      index: u32::MAX,
      generation: NonZero::new(u32::MAX).unwrap(),
      pool: self.pool,
    }
  }

  pub fn new_invalid_from_pool(pool: &'pool dyn DiagnosticPoolBuilder) -> DiagnosticReference<'pool> {
    DiagnosticReference {
      index: u32::MAX,
      generation: NonZero::new(u32::MAX).unwrap(),
      pool,
    }
  }

  pub fn create_physical_child(&self, offset: u64, size: Option<u64>, name: &str) -> DiagnosticReference<'pool> {
    self.pool.create(DiagnosticBranch::Physical { parent: self.dislocate(), offset }, size, name.into())
  }

  pub fn create_logical_child(&self, size: Option<u64>, branch_name: &'static str, name: &str) -> DiagnosticReference<'pool> {
    self.pool.create(
      DiagnosticBranch::Logical {
        parent: self.dislocate(),
        name: branch_name,
      },
      size,
      name,
    )
  }

  pub fn family_exists<P: DiagnosticPoolProvider>(&self, provider: &P) -> bool {
    if !self.exists(provider) {
      return false;
    }

    let parent = self.dereference(provider).unwrap().branch().parent();

    if parent.is_none() {
      return true;
    }

    let parent = parent.unwrap();

    parent.relocate(self.pool).family_exists(provider)
  }

  pub fn exists<P: DiagnosticPoolProvider>(&self, provider: &P) -> bool {
    self.dereference(provider).is_some()
  }

  pub fn dereference<'a, P: DiagnosticPoolProvider>(&self, provider: &'a P) -> Option<P::Node<'a>> {
    if !provider.was_built_by(self.pool) {
      return None;
    }

    provider.get(self.index, self.generation)
  }

  pub fn root<'a, P: DiagnosticPoolProvider>(&self, provider: &'a P) -> Option<P::Node<'a>> {
    let mut own = self.dereference(provider)?;

    while let Some(parent) = own.branch().parent().iter().flat_map(|p| p.relocate(self.pool).dereference(provider)).next() {
      own = parent
    }

    Some(own)
  }

  pub fn parent<'a, P: DiagnosticPoolProvider>(&self, provider: &'a P) -> Option<P::Node<'a>> {
    self.dereference(provider)?.branch().parent().map(|p| p.relocate(self.pool).dereference(provider)).flatten()
  }

  pub fn parent_reference<P: DiagnosticPoolProvider>(&self, provider: &P) -> Option<DiagnosticReference<'pool>> {
    self.dereference(provider)?.branch().parent().map(|p| p.relocate(self.pool))
  }

  pub fn parents<'provider, P: DiagnosticPoolProvider>(
    &self,
    provider: &'provider P,
  ) -> core::iter::Successors<P::Node<'provider>, impl (for<'a> FnMut(&'a P::Node<'provider>) -> Option<P::Node<'provider>>) + 'provider> {
    core::iter::successors(self.parent(provider), move |v| {
      v.branch().parent().map(move |v| v.relocate(provider.get_builder()).dereference(provider)).flatten()
    })
  }

  pub fn parents_incl_self<'provider, P: DiagnosticPoolProvider>(
    &self,
    provider: &'provider P,
  ) -> core::iter::Successors<P::Node<'provider>, impl (for<'a> FnMut(&'a P::Node<'provider>) -> Option<P::Node<'provider>>) + 'provider> {
    core::iter::successors(self.dereference(provider), |v| {
      v.branch().parent().map(|v| v.relocate(provider.get_builder()).dereference(provider)).flatten()
    })
  }

  pub fn dislocate(&self) -> DislocatedDiagnosticReference {
    DislocatedDiagnosticReference {
      index: self.index,
      generation: self.generation,
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DislocatedDiagnosticReference {
  pub(crate) index: u32,
  pub(crate) generation: NonZero<u32>,
}

impl DislocatedDiagnosticReference {
  pub fn relocate<'pl>(&self, pool: &'pl dyn DiagnosticPoolBuilder) -> DiagnosticReference<'pl> {
    DiagnosticReference {
      index: self.index,
      generation: self.generation,
      pool,
    }
  }
}

impl<'pool> Eq for DiagnosticReference<'pool> {}
impl<'pool> PartialEq for DiagnosticReference<'pool> {
  fn eq(&self, other: &Self) -> bool {
    self.index == other.index && self.generation == other.generation
  }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CompressedDislocatedDiagnosticReference(NonZero<u64>);

impl From<DislocatedDiagnosticReference> for CompressedDislocatedDiagnosticReference {
  fn from(value: DislocatedDiagnosticReference) -> Self {
    CompressedDislocatedDiagnosticReference(NonZero::new(((value.index as u64) << 32) | (value.generation.get() as u64)).unwrap())
  }
}

impl Into<DislocatedDiagnosticReference> for CompressedDislocatedDiagnosticReference {
  fn into(self) -> DislocatedDiagnosticReference {
    let index = (self.0.get() >> 32) as u32;
    let generation = NonZero::new((self.0.get() & 0xFFFFFFFF) as u32).unwrap();

    DislocatedDiagnosticReference { generation, index }
  }
}
