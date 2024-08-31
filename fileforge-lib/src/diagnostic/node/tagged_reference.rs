use super::reference::DiagnosticReference;

pub struct TaggedDiagnosticReference<'pool, const NODE_NAME_SIZE: usize, T> {
  reference: DiagnosticReference<'pool, NODE_NAME_SIZE>,
  value: T,
}

impl<'pool, const NODE_NAME_SIZE: usize, T> TaggedDiagnosticReference<'pool, NODE_NAME_SIZE, T> {
  pub fn tag(value: T, reference: DiagnosticReference<'pool, NODE_NAME_SIZE>) -> TaggedDiagnosticReference<'pool, NODE_NAME_SIZE, T> {
    TaggedDiagnosticReference {
      value,
      reference,
    }
  }

  pub fn reference(&self) -> DiagnosticReference<'pool, NODE_NAME_SIZE> {
    self.reference
  }

  pub fn value(&self) -> &T {
    &self.value
  }
}
