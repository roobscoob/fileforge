use core::cell::Cell;

use self::entry::DiagnosticPoolEntry;

use super::node::{branch::DiagnosticBranch, name::DiagnosticNodeName, reference::DiagnosticReference, DiagnosticNode};

pub mod field;
pub mod entry;

pub struct DiagnosticPool<'pool, const NODE_NAME_SIZE: usize> {
  contents: &'pool [DiagnosticPoolEntry<NODE_NAME_SIZE>],
  next_generation_to_write: Cell<u64>,
}

impl<'pool, const NODE_NAME_SIZE: usize> DiagnosticPool<'pool, NODE_NAME_SIZE> {
  pub fn new(over: &'pool mut [DiagnosticPoolEntry<NODE_NAME_SIZE>]) -> DiagnosticPool<'pool, NODE_NAME_SIZE> {
    DiagnosticPool {
      contents: over,
      next_generation_to_write: Cell::new(0),
    }
  }

  pub fn try_get(&self, index: usize, generation: u64) -> Option<DiagnosticNode<NODE_NAME_SIZE>> {
    self.contents.get(index)
      .map(|v| v.try_get(generation))
      .flatten()
  }

  pub fn expect_get(&self, index: usize, generation: u64, message: &str) -> DiagnosticNode<NODE_NAME_SIZE> {
    if let Some(x) = self.contents.get(index).map(|v| v.expect_get(generation, message)) {
      return x;
    }

    panic!("Diagnostic Expectation Failed: {message}")
  }

  fn get_space_to_consume(&self, keeping_tree: Option<DiagnosticNode<NODE_NAME_SIZE>>) -> Option<(&DiagnosticPoolEntry<NODE_NAME_SIZE>, usize)> {
    let mut lowest_generation_index_pair: Option<(u64, usize)> = None;

    for (entry, index) in self.contents.iter().zip(0usize..) {
      match (entry.get(), lowest_generation_index_pair) {
        (None, _) =>
          { lowest_generation_index_pair = Some((0, index)); break }

        (Some(field), None)
          if !field.contents.is_family_of(keeping_tree.as_ref(), self) =>
          lowest_generation_index_pair = Some((field.generation(), index)),

        (Some(field), Some(existing))
          if !field.contents.is_family_of(keeping_tree.as_ref(), self)
          && existing.0 > field.generation() =>
          lowest_generation_index_pair = Some((field.generation(), index)),

        _ => continue,
      }
    }

    lowest_generation_index_pair
      .map(|v| self.contents.get(v.1).map(|e| (e, v.1)))
      .flatten()
  }

  fn increment_generation(&self) -> u64 {
    let v = self.next_generation_to_write.take();
    self.next_generation_to_write.set(v + 1);
    v
  }

  pub fn try_create(&self, branch: DiagnosticBranch, size: u64, name: DiagnosticNodeName<NODE_NAME_SIZE>) -> DiagnosticReference<NODE_NAME_SIZE> {
    match self.get_space_to_consume(branch.parent().map(|r| r.relocate(self).dereference()).flatten()) {
      None => DiagnosticReference { index: self.contents.len() + 1, generation: 0, pool: self },
      Some((v, i)) => {
        let new_generation = self.increment_generation();
        v.write(DiagnosticNode { branch, size, name }, new_generation);
        DiagnosticReference { index: i, generation: new_generation, pool: self }
      }
    }
  }
}