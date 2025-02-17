use crate::diagnostic::{
  node::reference::{CompressedDislocatedDiagnosticReference, DiagnosticReference, DislocatedDiagnosticReference},
  pool::DiagnosticPool,
  value::DiagnosticValue,
};

pub enum DiagnosticKind {
  Reader,
  ReaderLength,
  ReaderPosition,
}

pub struct DiagnosticStore<'pool, const NODE_NAME_SIZE: usize> {
  reader: Option<CompressedDislocatedDiagnosticReference>,
  reader_length: Option<CompressedDislocatedDiagnosticReference>,
  reader_position: Option<CompressedDislocatedDiagnosticReference>,

  pool: Option<&'pool dyn DiagnosticPool<NODE_NAME_SIZE>>,
}

impl<'pool, const NODE_NAME_SIZE: usize> DiagnosticStore<'pool, NODE_NAME_SIZE> {
  pub fn new() -> Self {
    Self {
      reader: None,
      reader_length: None,
      reader_position: None,
      pool: None,
    }
  }

  pub fn get(&self, kind: DiagnosticKind) -> Option<DiagnosticReference<'pool, NODE_NAME_SIZE>> {
    match kind {
      DiagnosticKind::Reader => self.reader,
      DiagnosticKind::ReaderLength => self.reader_length,
      DiagnosticKind::ReaderPosition => self.reader_position,
    }
    .map(|v| Into::<DislocatedDiagnosticReference>::into(v).relocate(self.pool.unwrap()))
  }

  pub fn infuse<T>(&self, kind: DiagnosticKind, value: T) -> DiagnosticValue<'pool, T, NODE_NAME_SIZE> { DiagnosticValue(value, self.get(kind)) }

  pub fn set(&mut self, kind: DiagnosticKind, reference: DiagnosticReference<'pool, NODE_NAME_SIZE>) {
    self.pool = Some(reference.pool);

    match kind {
      DiagnosticKind::Reader => self.reader = Some(reference.dislocate().into()),
      DiagnosticKind::ReaderLength => self.reader_length = Some(reference.dislocate().into()),
      DiagnosticKind::ReaderPosition => self.reader_position = Some(reference.dislocate().into()),
    }
  }
}
