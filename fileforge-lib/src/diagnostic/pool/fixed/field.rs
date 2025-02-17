use core::num::NonZero;

use crate::diagnostic::node::DiagnosticNode;

#[derive(Clone, Copy, Debug)]
pub struct DiagnosticPoolField<const NODE_NAME_SIZE: usize> {
  pub(crate) generation: NonZero<u32>,
  pub(crate) contents: DiagnosticNode<NODE_NAME_SIZE>,
}

impl<const NODE_NAME_SIZE: usize> DiagnosticPoolField<NODE_NAME_SIZE> {
  pub fn try_get(&self, generation: NonZero<u32>) -> Option<DiagnosticNode<NODE_NAME_SIZE>> {
    if generation != self.generation {
      return None;
    }

    Some(self.contents)
  }

  pub fn generation(&self) -> NonZero<u32> { self.generation }
}
