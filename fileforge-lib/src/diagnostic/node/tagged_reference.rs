use super::reference::DiagnosticReference;

#[derive(Clone)]
pub struct TaggedDiagnosticReference<'pool, T: Clone> {
  reference: DiagnosticReference<'pool>,
  value: T,
}

impl<'pool, T: Clone>
  TaggedDiagnosticReference<'pool, T>
{
  pub fn tag(
    value: T,
    reference: DiagnosticReference<'pool>,
  ) -> TaggedDiagnosticReference<'pool, T> {
    TaggedDiagnosticReference { value, reference }
  }

  pub fn reference(&self) -> DiagnosticReference<'pool> { self.reference }

  pub fn value(&self) -> &T { &self.value }
}
