use crate::diagnostic::{
  node::reference::{CompressedDislocatedDiagnosticReference, DiagnosticReference, DislocatedDiagnosticReference},
  pool::DiagnosticPoolBuilder,
  value::DiagnosticValue,
};

pub enum DiagnosticKind {
  Reader,
  ReaderLength,
  ReaderPosition,
}

#[derive(Clone, Copy)]
pub struct DiagnosticStore<'pool> {
  reader: Option<CompressedDislocatedDiagnosticReference>,
  reader_length: Option<CompressedDislocatedDiagnosticReference>,
  reader_position: Option<CompressedDislocatedDiagnosticReference>,

  pool: Option<&'pool dyn DiagnosticPoolBuilder>,
}

impl<'pool> DiagnosticStore<'pool> {
  pub fn new() -> Self {
    Self {
      reader: None,
      reader_length: None,
      reader_position: None,
      pool: None,
    }
  }

  pub fn get(&self, kind: DiagnosticKind) -> Option<DiagnosticReference<'pool>> {
    match kind {
      DiagnosticKind::Reader => self.reader,
      DiagnosticKind::ReaderLength => self.reader_length,
      DiagnosticKind::ReaderPosition => self.reader_position,
    }
    .map(|v| Into::<DislocatedDiagnosticReference>::into(v).relocate(self.pool.unwrap()))
  }

  pub fn infuse<T>(&self, kind: DiagnosticKind, value: T) -> DiagnosticValue<'pool, T> {
    DiagnosticValue(value, self.get(kind))
  }

  pub fn set(&mut self, kind: DiagnosticKind, reference: Option<DiagnosticReference<'pool>>) {
    if let Some(reference) = reference {
      self.pool = Some(reference.pool);

      match kind {
        DiagnosticKind::Reader => self.reader = Some(reference.dislocate().into()),
        DiagnosticKind::ReaderLength => self.reader_length = Some(reference.dislocate().into()),
        DiagnosticKind::ReaderPosition => self.reader_position = Some(reference.dislocate().into()),
      }
    }
  }

  pub fn with(mut self, kind: DiagnosticKind, reference: Option<DiagnosticReference<'pool>>) -> Self {
    self.set(kind, reference);
    self
  }
}
