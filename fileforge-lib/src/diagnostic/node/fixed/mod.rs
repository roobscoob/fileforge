use name::FixedDiagnosticNodeName;

use super::{branch::DiagnosticBranch, name::DiagnosticNodeName, DiagnosticNode};

pub mod name;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FixedDiagnosticNode<const NAME_SIZE: usize> {
  pub branch: DiagnosticBranch,
  pub size: Option<u64>,
  pub name: FixedDiagnosticNodeName<NAME_SIZE>,
}

impl<const NAME_SIZE: usize> DiagnosticNode for FixedDiagnosticNode<NAME_SIZE> {
  fn branch(&self) -> &DiagnosticBranch { &self.branch }
  fn name(&self) -> &dyn DiagnosticNodeName { &self.name }
  fn size(&self) -> Option<u64> { self.size }
}