use core::num::NonZero;

use crate::diagnostic::node::fixed::FixedDiagnosticNode;

#[derive(Clone, Copy, Debug)]
pub struct FixedDiagnosticPoolField<const NODE_NAME_SIZE: usize> {
  pub(crate) generation: NonZero<u32>,
  pub(crate) contents: FixedDiagnosticNode<NODE_NAME_SIZE>,
}

impl<const NODE_NAME_SIZE: usize> FixedDiagnosticPoolField<NODE_NAME_SIZE> {
  pub fn try_get(&self, generation: NonZero<u32>) -> Option<FixedDiagnosticNode<NODE_NAME_SIZE>> {
    if generation != self.generation {
      return None;
    }

    Some(self.contents)
  }

  pub fn generation(&self) -> NonZero<u32> { self.generation }
}
