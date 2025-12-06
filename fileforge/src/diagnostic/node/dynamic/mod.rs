use super::{branch::DiagnosticBranch, name::DiagnosticNodeName, DiagnosticNode};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DynamicDiagnosticNode {
  pub branch: DiagnosticBranch,
  pub size: Option<u64>,
  pub name: alloc::string::String,
}

impl DiagnosticNode for DynamicDiagnosticNode {
  fn branch(&self) -> &DiagnosticBranch {
    &self.branch
  }

  fn name(&self) -> &dyn DiagnosticNodeName {
    &self.name
  }

  fn size(&self) -> Option<u64> {
    self.size
  }
}

impl DiagnosticNodeName for alloc::string::String {
  fn as_str(&self) -> &str {
    self
  }

  fn show_ellipsis(&self) -> bool {
    false
  }
}
