use super::reference::DiagnosticReference;

#[derive(Clone)]
pub struct TaggedDiagnosticReference<'pool, const NODE_NAME_SIZE: usize, T: Clone> {
  reference: DiagnosticReference<'pool, NODE_NAME_SIZE>,
  value: T,
}

impl<'pool, const NODE_NAME_SIZE: usize, T: Clone>
  TaggedDiagnosticReference<'pool, NODE_NAME_SIZE, T>
{
  pub fn tag(
    value: T,
    reference: DiagnosticReference<'pool, NODE_NAME_SIZE>,
  ) -> TaggedDiagnosticReference<'pool, NODE_NAME_SIZE, T> {
    TaggedDiagnosticReference { value, reference }
  }

  pub fn reference(&self) -> DiagnosticReference<'pool, NODE_NAME_SIZE> { self.reference }

  pub fn value(&self) -> &T { &self.value }
}
