pub mod branch;
pub mod fixed;
pub mod name;
pub mod reference;
pub mod tagged_reference;

#[cfg(feature = "alloc")]
pub mod dynamic;

use core::fmt::Debug;

use branch::DiagnosticBranch;
use name::DiagnosticNodeName;

pub trait DiagnosticNode: Debug {
  fn branch(&self) -> &DiagnosticBranch;

  fn name(&self) -> &dyn DiagnosticNodeName;

  fn size(&self) -> Option<u64>;

  fn eq(&self, other: &dyn DiagnosticNode) -> bool {
    other.branch() == self.branch() && other.name().as_str() == self.name().as_str() && other.name().show_ellipsis() == self.name().show_ellipsis() && other.size() == self.size()
  }
}

impl<'a, N: DiagnosticNode> DiagnosticNode for core::cell::Ref<'a, N> {
  fn branch(&self) -> &DiagnosticBranch {
    (**self).branch()
  }

  fn name(&self) -> &dyn DiagnosticNodeName {
    (**self).name()
  }

  fn size(&self) -> Option<u64> {
    (**self).size()
  }
}
