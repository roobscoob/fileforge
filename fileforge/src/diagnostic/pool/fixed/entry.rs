use core::{cell::Cell, num::NonZero};

use crate::diagnostic::node::fixed::FixedDiagnosticNode;

use super::field::FixedDiagnosticPoolField;

#[derive(Default, Clone, Debug)]
pub struct FixedDiagnosticPoolEntry<const NODE_NAME_SIZE: usize> {
  field: Cell<Option<FixedDiagnosticPoolField<NODE_NAME_SIZE>>>,
}

impl<const NODE_NAME_SIZE: usize> FixedDiagnosticPoolEntry<NODE_NAME_SIZE> {
  pub fn get(&self) -> Option<FixedDiagnosticPoolField<NODE_NAME_SIZE>> { self.field.get() }
  pub fn write(&self, node: FixedDiagnosticNode<NODE_NAME_SIZE>, generation: NonZero<u32>) { self.field.set(Some(FixedDiagnosticPoolField { generation, contents: node })) }
}
