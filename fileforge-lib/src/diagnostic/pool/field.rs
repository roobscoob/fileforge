use crate::diagnostic::node::DiagnosticNode;

#[derive(Clone, Copy, Debug)]
pub struct DiagnosticPoolField<const NODE_NAME_SIZE: usize> {
  pub(crate) generation: u64,
  pub(crate) contents: DiagnosticNode<NODE_NAME_SIZE>,
}

impl<const NODE_NAME_SIZE: usize> DiagnosticPoolField<NODE_NAME_SIZE> {
  pub fn try_get(&self, generation: u64) -> Option<DiagnosticNode<NODE_NAME_SIZE>> {
    if generation != self.generation {
      return None;
    }

    Some(self.contents)
  }

  pub fn expect_get(&self, generation: u64, message: &str) -> DiagnosticNode<NODE_NAME_SIZE> {
    if generation != self.generation {
      panic!("Diagnostic Expectation Failed: {message}")
    }

    self.contents
  }

  pub fn generation(&self) -> u64 { self.generation }
}
