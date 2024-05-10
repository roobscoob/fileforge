use self::{branch::DiagnosticBranch, name::DiagnosticNodeName};

use super::pool::DiagnosticPool;

pub mod name;
pub mod branch;
pub mod reference;
pub mod tagged_reference;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticNode<const NAME_SIZE: usize> {
  pub(crate) branch: DiagnosticBranch,
  pub(crate) size: u64,
  pub(crate) name: DiagnosticNodeName<NAME_SIZE>,
}

impl<const NAME_SIZE: usize> DiagnosticNode<NAME_SIZE> {
  pub fn is_family_of<'l>(&self, other: Option<&DiagnosticNode<NAME_SIZE>>, pool: &DiagnosticPool<'l, NAME_SIZE>) -> bool {
    if other.is_none() {
      return false;
    }

    let other = other.unwrap();

    if other == self { return true }

    let parent = other.branch.parent();

    if parent.is_none() {
      return false;
    }

    let parent = parent.unwrap();

    parent.relocate(pool).parents().any(|p| p == *self)
  }
}