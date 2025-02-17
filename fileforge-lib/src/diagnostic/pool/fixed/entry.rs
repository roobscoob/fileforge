use core::{cell::Cell, num::NonZero};

use crate::diagnostic::node::DiagnosticNode;

use super::field::DiagnosticPoolField;

#[derive(Default, Clone, Debug)]
pub struct DiagnosticPoolEntry<const NODE_NAME_SIZE: usize> {
  field: Cell<Option<DiagnosticPoolField<NODE_NAME_SIZE>>>,
}

impl<const NODE_NAME_SIZE: usize> DiagnosticPoolEntry<NODE_NAME_SIZE> {
  pub fn get(&self) -> Option<DiagnosticPoolField<NODE_NAME_SIZE>> { self.field.get() }

  pub fn write(&self, node: DiagnosticNode<NODE_NAME_SIZE>, generation: NonZero<u32>) { self.field.set(Some(DiagnosticPoolField { generation, contents: node })) }
}
