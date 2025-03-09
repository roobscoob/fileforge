use core::{cell::Cell, num::NonZero};

use crate::diagnostic::node::fixed::FixedDiagnosticNode;

use self::entry::FixedDiagnosticPoolEntry;

use super::{super::node::{branch::DiagnosticBranch, reference::DiagnosticReference}, DiagnosticPoolBuilder, DiagnosticPoolProvider};

pub mod entry;
pub mod field;

pub struct FixedDiagnosticPool<'pool, const NODE_NAME_SIZE: usize> {
  contents: &'pool [FixedDiagnosticPoolEntry<NODE_NAME_SIZE>],
  next_generation_to_write: Cell<u32>,
}

impl<'pool, const NODE_NAME_SIZE: usize> FixedDiagnosticPool<'pool, NODE_NAME_SIZE> {
  pub fn new(over: &'pool mut [FixedDiagnosticPoolEntry<NODE_NAME_SIZE>]) -> FixedDiagnosticPool<'pool, NODE_NAME_SIZE> {
    FixedDiagnosticPool {
      contents: over,
      next_generation_to_write: Cell::new(2),
    }
  }

  fn is_family_of(
    &self,
    check: &FixedDiagnosticNode<NODE_NAME_SIZE>,
    against: Option<&FixedDiagnosticNode<NODE_NAME_SIZE>>,
  ) -> bool {
    if against.is_none() {
      return false;
    }

    let against = against.unwrap();

    if against == check {
      return true;
    }

    let parent = against.branch.parent();

    if parent.is_none() {
      return false;
    }

    let parent = parent.unwrap();

    parent.relocate(self).parents(self).any(|p| p == *check)
  }

  fn get_space_to_consume(&self, keeping_tree: Option<FixedDiagnosticNode<NODE_NAME_SIZE>>) -> Option<(&FixedDiagnosticPoolEntry<NODE_NAME_SIZE>, usize)> {
    let mut lowest_generation_index_pair: Option<(NonZero<u32>, usize)> = None;

    for (entry, index) in self.contents.iter().zip(0usize..) {
      match (entry.get(), lowest_generation_index_pair) {
        (None, _) => {
          lowest_generation_index_pair = Some((NonZero::new(1).unwrap(), index));
          break;
        }

        (Some(field), None) if !self.is_family_of(&field.contents, keeping_tree.as_ref()) => lowest_generation_index_pair = Some((field.generation(), index)),

        (Some(field), Some(existing)) if !self.is_family_of(&field.contents, keeping_tree.as_ref()) && existing.0 > field.generation() => {
          lowest_generation_index_pair = Some((field.generation(), index))
        }

        _ => continue,
      }
    }

    lowest_generation_index_pair.map(|v| self.contents.get(v.1).map(|e| (e, v.1))).flatten()
  }

  fn increment_generation(&self) -> u32 {
    let v = self.next_generation_to_write.take();
    self.next_generation_to_write.set(v + 1);
    v
  }
}

impl<'pool, const NODE_NAME_SIZE: usize> DiagnosticPoolProvider for FixedDiagnosticPool<'pool, NODE_NAME_SIZE> {
  type Node = FixedDiagnosticNode<NODE_NAME_SIZE>;

  fn get(&self, index: u32, generation: NonZero<u32>) -> Option<Self::Node> { self.contents.get(index as usize).and_then(|v| v.get()).and_then(|v| v.try_get(generation)) }

  fn was_built_by(&self, _: &dyn DiagnosticPoolBuilder) -> bool {
    // TODO: This doesn't support multiple DPPs
    
    true
  }

  fn get_builder(&self) -> &dyn DiagnosticPoolBuilder {
    self
  }
}

impl<'pool, const NODE_NAME_SIZE: usize> DiagnosticPoolBuilder for FixedDiagnosticPool<'pool, NODE_NAME_SIZE> {
  fn create(&self, branch: DiagnosticBranch, size: Option<u64>, name: &str) -> DiagnosticReference {
    match self.get_space_to_consume(branch.parent().map(|r| r.relocate(self).dereference(self)).flatten()) {
      None => DiagnosticReference {
        index: (self.contents.len() + 1) as u32,
        generation: NonZero::new(1u32).unwrap(),
        pool: self,
      },
      Some((v, i)) => {
        let new_generation = NonZero::new(self.increment_generation()).unwrap();
        v.write(FixedDiagnosticNode { branch, size, name: name.into() }, new_generation);
        DiagnosticReference {
          index: i as u32,
          generation: new_generation,
          pool: self,
        }
      }
    }
  }
}
