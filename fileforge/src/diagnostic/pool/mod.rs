pub mod fixed;

use core::num::NonZero;

use super::node::{branch::DiagnosticBranch, reference::DiagnosticReference, DiagnosticNode};

pub trait DiagnosticPoolProvider {
  type Node: DiagnosticNode;

  fn get(&self, index: u32, generation: NonZero<u32>) -> Option<Self::Node>;

  fn was_built_by(&self, builder: &dyn DiagnosticPoolBuilder) -> bool;

  fn get_builder(&self) -> &dyn DiagnosticPoolBuilder;
}

pub trait DiagnosticPoolBuilder {
  fn create(&self, branch: DiagnosticBranch, size: Option<u64>, name: &str) -> DiagnosticReference;
}
