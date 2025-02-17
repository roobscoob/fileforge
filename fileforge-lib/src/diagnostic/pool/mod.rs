pub mod fixed;

use core::num::NonZero;

use super::node::{branch::DiagnosticBranch, name::DiagnosticNodeName, reference::DiagnosticReference, DiagnosticNode};

pub trait DiagnosticPool<const NODE_NAME_SIZE: usize> {
  fn get(&self, index: u32, generation: NonZero<u32>) -> Option<DiagnosticNode<NODE_NAME_SIZE>>;
  fn create(&self, branch: DiagnosticBranch, size: Option<u64>, name: DiagnosticNodeName<NODE_NAME_SIZE>) -> DiagnosticReference<NODE_NAME_SIZE>;
}
