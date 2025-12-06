use core::num::NonZero;

use crate::diagnostic::node::dynamic::DynamicDiagnosticNode;

#[derive(Clone, Debug)]
pub struct DynamicDiagnosticPoolField {
  pub(crate) generation: NonZero<u32>,
  pub(crate) contents: DynamicDiagnosticNode,
}

impl DynamicDiagnosticPoolField {
  pub fn try_get(&self, generation: NonZero<u32>) -> Option<&DynamicDiagnosticNode> {
    if generation != self.generation {
      return None;
    }

    Some(&self.contents)
  }

  pub fn generation(&self) -> NonZero<u32> {
    self.generation
  }
}
