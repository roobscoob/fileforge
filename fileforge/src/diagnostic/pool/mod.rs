pub mod dynamic;
pub mod fixed;

use core::num::NonZero;

use super::node::{branch::DiagnosticBranch, reference::DiagnosticReference, DiagnosticNode};

pub trait DiagnosticPoolProvider {
  type Node<'a>: DiagnosticNode + 'a
  where
    Self: 'a;

  fn get<'a>(&'a self, index: u32, generation: NonZero<u32>) -> Option<Self::Node<'a>>;

  fn was_built_by(&self, builder: &dyn DiagnosticPoolBuilder) -> bool;

  fn get_builder(&self) -> &dyn DiagnosticPoolBuilder;
}

impl<P: DiagnosticPoolProvider> DiagnosticPoolProvider for &P {
  type Node<'a>
    = P::Node<'a>
  where
    Self: 'a;

  fn get<'a>(&'a self, index: u32, generation: NonZero<u32>) -> Option<Self::Node<'a>> {
    (**self).get(index, generation)
  }

  fn was_built_by(&self, builder: &dyn DiagnosticPoolBuilder) -> bool {
    (**self).was_built_by(builder)
  }

  fn get_builder(&self) -> &dyn DiagnosticPoolBuilder {
    (**self).get_builder()
  }
}

pub trait DiagnosticPoolBuilder {
  fn create<'a>(&'a self, branch: DiagnosticBranch, size: Option<u64>, name: &str) -> DiagnosticReference<'a>;
}
