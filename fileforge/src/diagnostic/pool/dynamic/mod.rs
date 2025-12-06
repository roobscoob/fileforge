pub mod field;

extern crate alloc;

use alloc::vec::Vec;
use core::{cell::RefCell, num::NonZero, str::FromStr};

use crate::diagnostic::{
  node::{branch::DiagnosticBranch, dynamic::DynamicDiagnosticNode, reference::DiagnosticReference},
  pool::{dynamic::field::DynamicDiagnosticPoolField, DiagnosticPoolBuilder, DiagnosticPoolProvider},
};

pub struct DynamicDiagnosticPool {
  contents: RefCell<Vec<DynamicDiagnosticPoolField>>,
  next_generation_to_write: RefCell<u32>,
  max_capacity: Option<usize>, // None = unlimited growth
}

impl DynamicDiagnosticPool {
  /// Create a pool that can grow indefinitely
  pub fn new() -> Self {
    DynamicDiagnosticPool {
      contents: RefCell::new(Vec::new()),
      next_generation_to_write: RefCell::new(2),
      max_capacity: None,
    }
  }

  /// Create a pool with a maximum capacity (will reuse slots after hitting limit)
  pub fn with_capacity(capacity: usize) -> Self {
    DynamicDiagnosticPool {
      contents: RefCell::new(Vec::with_capacity(capacity)),
      next_generation_to_write: RefCell::new(2),
      max_capacity: Some(capacity),
    }
  }

  /// Create a pool from existing entries with optional max capacity
  pub fn with_contents(contents: Vec<DynamicDiagnosticPoolField>, max_capacity: Option<usize>) -> Self {
    DynamicDiagnosticPool {
      contents: RefCell::new(contents),
      next_generation_to_write: RefCell::new(2),
      max_capacity,
    }
  }

  fn can_grow(&self) -> bool {
    match self.max_capacity {
      None => true, // Unlimited growth
      Some(max) => self.contents.borrow().len() < max,
    }
  }

  fn increment_generation(&self) -> u32 {
    let mut gen = self.next_generation_to_write.borrow_mut();
    let v = *gen;
    *gen = v + 1;
    v
  }

  fn get_space_to_consume(&self, keeping_tree: Option<DynamicDiagnosticNode>) -> Option<usize> {
    let contents = self.contents.borrow();
    let mut lowest_generation_index_pair: Option<(NonZero<u32>, usize)> = None;

    for (index, field) in contents.iter().enumerate() {
      if !self.is_family_of(&field.contents, keeping_tree.as_ref()) {
        if let Some(lowest_generation_index_pair) = lowest_generation_index_pair {
        } else {
        }
      }

      //   match (entry, lowest_generation_index_pair) {
      //     (field, None) if !self.is_family_of(&field.contents, keeping_tree.as_ref()) => lowest_generation_index_pair = Some((field.generation(), index)),

      //     (field, Some(existing)) if !self.is_family_of(&field.contents, keeping_tree.as_ref()) && existing.0 > field.generation() => lowest_generation_index_pair = Some((field.generation(), index)),

      //     _ => continue,
      //   }
    }

    lowest_generation_index_pair.map(|v| v.1)
  }
}

impl DiagnosticPoolProvider for DynamicDiagnosticPool {
  type Node<'a> = core::cell::Ref<'a, DynamicDiagnosticNode>;

  fn get<'a>(&'a self, index: u32, generation: NonZero<u32>) -> Option<Self::Node<'a>> {
    core::cell::Ref::filter_map(self.contents.borrow(), |v| v.get(index as usize).and_then(|v| v.try_get(generation))).ok()
  }

  fn get_builder(&self) -> &dyn DiagnosticPoolBuilder {
    self
  }

  fn was_built_by(&self, other: &dyn DiagnosticPoolBuilder) -> bool {
    // TODO: This doesn't support multiple DPPs

    true
  }
}

impl DiagnosticPoolBuilder for DynamicDiagnosticPool {
  fn create<'a>(&'a self, branch: DiagnosticBranch, size: Option<u64>, name: &str) -> DiagnosticReference<'a> {
    let node = DynamicDiagnosticNode {
      branch,
      size,
      name: String::from(name),
    };

    // Strategy: Try to grow first, only reuse slots if we can't
    if self.can_grow() {
      // Happy path: just append a new entry (no generations needed!)
      let mut contents = self.contents.borrow_mut();
      let index = contents.len();
      let generation = NonZero::new(1).unwrap();

      contents.push(DynamicDiagnosticPoolField {
        contents: DynamicDiagnosticNode {
          branch,
          size,
          name: alloc::string::String::from_str(name).unwrap(),
        },
        generation,
      });

      DiagnosticReference {
        index: index as u32,
        generation,
        pool: self,
      }
    } else {
      // Fallback: reuse existing slot (generations protect against stale references)
      let parent_node = branch.parent().map(|r| r.relocate(self).dereference(self)).flatten();

      match self.get_space_to_consume(parent_node) {
        None => {
          // Pool is full and no slots can be reused - return invalid reference
          let contents = self.contents.borrow();
          DiagnosticReference {
            index: (contents.len() + 1) as u32,
            generation: NonZero::new(1u32).unwrap(),
            pool: self,
          }
        }
        Some(index) => {
          let new_generation = NonZero::new(self.increment_generation()).unwrap();

          let contents = self.contents.borrow();
          if let Some(entry) = contents.get(index) {
            entry.write(node, new_generation);
          }

          DiagnosticReference {
            index: index as u32,
            generation: new_generation,
            pool: self,
          }
        }
      }
    }
  }
}
