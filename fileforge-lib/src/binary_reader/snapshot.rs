use crate::{
  binary_reader::{diagnostic_store::DiagnosticStore, endianness::Endianness},
  stream::RestorableStream,
};

pub struct BinaryReaderSnapshot<'pool, S: RestorableStream<Type = u8>> {
  pub(super) snapshot: S::Snapshot,
  pub(super) endianness: Endianness,
  pub(super) diagnostics: DiagnosticStore<'pool>,
}

impl<'pool, S: RestorableStream<Type = u8>> Clone for BinaryReaderSnapshot<'pool, S>
where
  S::Snapshot: Clone,
{
  fn clone(&self) -> Self {
    Self {
      diagnostics: self.diagnostics,
      endianness: self.endianness,
      snapshot: self.snapshot.clone(),
    }
  }
}
