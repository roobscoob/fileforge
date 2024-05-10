use core::cell::Cell;

use crate::diagnostic::node::DiagnosticNode;

use super::field::DiagnosticPoolField;

#[derive(Default, Clone)]
pub struct DiagnosticPoolEntry<const NODE_NAME_SIZE: usize> {
  field: Cell<Option<DiagnosticPoolField<NODE_NAME_SIZE>>>,
}

impl<const NODE_NAME_SIZE: usize> DiagnosticPoolEntry<NODE_NAME_SIZE> {
  pub fn try_get(&self, generation: u64) -> Option<DiagnosticNode<NODE_NAME_SIZE>> {
    self.field.get()
      .map(|v| v.try_get(generation))
      .flatten()
  }

  pub fn expect_get(&self, generation: u64, message: &str) -> DiagnosticNode<NODE_NAME_SIZE> {
    if let Some(v) = self.field.get().map(|v| v.expect_get(generation, message)) {
      return v;
    }

    panic!("Diagnostic Expectation Failed: {message}")
  }

  pub fn get(&self) -> Option<DiagnosticPoolField<NODE_NAME_SIZE>> {
    self.field.get()
  }

  pub fn write(&self, node: DiagnosticNode<NODE_NAME_SIZE>, generation: u64) {
    self.field.set(Some(DiagnosticPoolField { generation, contents: node }))
  }
}