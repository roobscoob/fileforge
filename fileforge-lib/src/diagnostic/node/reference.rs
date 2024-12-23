use core::{fmt::Debug, u64, usize};

use crate::diagnostic::pool::DiagnosticPool;

use super::{branch::DiagnosticBranch, name::DiagnosticNodeName, DiagnosticNode};

#[derive(Clone, Copy)]
pub struct DiagnosticReference<'pool, const NODE_NAME_SIZE: usize> {
  pub(crate) index: usize,
  pub(crate) generation: u64,
  pub(crate) pool: &'pool DiagnosticPool<'pool, NODE_NAME_SIZE>,
}

impl<'pool, const NODE_NAME_SIZE: usize> Debug for DiagnosticReference<'pool, NODE_NAME_SIZE> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    Debug::fmt(&self.dereference(), f)
  }
}

impl<'pool, const NODE_NAME_SIZE: usize> DiagnosticReference<'pool, NODE_NAME_SIZE> {
  pub fn exists(&self) -> bool { self.pool.try_get(self.index, self.generation).is_some() }

  pub fn new_invalid(&self) -> DiagnosticReference<'pool, NODE_NAME_SIZE> {
    DiagnosticReference {
      index: usize::MAX,
      generation: u64::MAX,
      pool: self.pool,
    }
  }

  pub fn new_invalid_from_pool(
    pool: &'pool DiagnosticPool<'pool, NODE_NAME_SIZE>,
  ) -> DiagnosticReference<'pool, NODE_NAME_SIZE> {
    DiagnosticReference {
      index: usize::MAX,
      generation: u64::MAX,
      pool,
    }
  }

  pub fn family_exists(&self) -> bool {
    if !self.exists() {
      return false;
    }

    let parent = self.dereference().unwrap().branch.parent();

    if parent.is_none() {
      return true;
    }

    let parent = parent.unwrap();

    parent.relocate(self.pool).family_exists()
  }

  pub fn dereference(&self) -> Option<DiagnosticNode<NODE_NAME_SIZE>> {
    self.pool.try_get(self.index, self.generation)
  }

  pub fn dereference_expect(&self, message: &str) -> DiagnosticNode<NODE_NAME_SIZE> {
    self.pool.expect_get(self.index, self.generation, message)
  }

  pub fn root(&self) -> Option<DiagnosticNode<NODE_NAME_SIZE>> {
    let own = self.dereference()?;
    let own_parent = own.branch.parent().map(|p| p.relocate(self.pool));

    if let Some(parent) = own_parent {
      parent.root()
    } else {
      Some(own)
    }
  }

  pub fn parent(&self) -> Option<DiagnosticNode<NODE_NAME_SIZE>> {
    self
      .dereference()?
      .branch
      .parent()
      .map(|p| p.relocate(self.pool).dereference())
      .flatten()
  }

  pub fn parent_reference(&self) -> Option<DiagnosticReference<NODE_NAME_SIZE>> {
    self
      .dereference()?
      .branch
      .parent()
      .map(|p| p.relocate(self.pool))
  }

  pub fn expect_root(&self, message: &str) -> DiagnosticNode<NODE_NAME_SIZE> {
    let own = self.dereference_expect(message);
    let own_parent = own.branch.parent().map(|p| p.relocate(self.pool));

    if let Some(parent) = own_parent {
      parent.expect_root(message)
    } else {
      own
    }
  }

  pub fn expect_parent(&self, message: &str) -> Option<DiagnosticNode<NODE_NAME_SIZE>> {
    self
      .dereference_expect(message)
      .branch
      .parent()
      .map(|p| p.relocate(self.pool).dereference_expect(message))
  }

  pub fn create_physical_child(
    &self,
    offset: u64,
    size: Option<u64>,
    name: DiagnosticNodeName<NODE_NAME_SIZE>,
  ) -> DiagnosticReference<'pool, NODE_NAME_SIZE> {
    if !self.exists() {
      return self.new_invalid();
    }

    self.pool.try_create(
      DiagnosticBranch::Physical {
        parent: self.dislocate(),
        offset,
      },
      size,
      name,
    )
  }

  pub fn create_logical_child(
    &self,
    size: Option<u64>,
    branch_name: &'static str,
    name: DiagnosticNodeName<NODE_NAME_SIZE>,
  ) -> DiagnosticReference<'pool, NODE_NAME_SIZE> {
    if !self.exists() {
      return self.new_invalid();
    }

    self.pool.try_create(
      DiagnosticBranch::Logical {
        parent: self.dislocate(),
        name: branch_name,
      },
      size,
      name,
    )
  }

  pub fn parents<'capture>(
    &'capture self,
  ) -> core::iter::Successors<
    DiagnosticNode<NODE_NAME_SIZE>,
    impl FnMut(&DiagnosticNode<NODE_NAME_SIZE>) -> Option<DiagnosticNode<NODE_NAME_SIZE>> + 'capture,
  > {
    core::iter::successors(self.parent(), |v| {
      v.branch
        .parent()
        .map(|v| v.relocate(self.pool).dereference())
        .flatten()
    })
  }

  pub fn parents_incl_self<'capture>(
    &'capture self,
  ) -> core::iter::Successors<
    DiagnosticNode<NODE_NAME_SIZE>,
    impl FnMut(&DiagnosticNode<NODE_NAME_SIZE>) -> Option<DiagnosticNode<NODE_NAME_SIZE>> + 'capture,
  > {
    core::iter::successors(self.dereference(), |v| {
      v.branch
        .parent()
        .map(|v| v.relocate(self.pool).dereference())
        .flatten()
    })
  }

  pub fn expect_parents<'capture>(
    &'capture self,
    message: &'capture str,
  ) -> core::iter::Successors<
    DiagnosticNode<NODE_NAME_SIZE>,
    impl FnMut(&DiagnosticNode<NODE_NAME_SIZE>) -> Option<DiagnosticNode<NODE_NAME_SIZE>> + 'capture,
  > {
    core::iter::successors(self.expect_parent(message), |v| {
      v.branch
        .parent()
        .map(|v| v.relocate(self.pool).dereference_expect(message))
    })
  }

  pub fn dislocate(&self) -> DislocatedDiagnosticReference {
    DislocatedDiagnosticReference {
      index: self.index,
      generation: self.generation,
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DislocatedDiagnosticReference {
  pub(crate) index: usize,
  pub(crate) generation: u64,
}

impl DislocatedDiagnosticReference {
  pub fn relocate<'pl, const NODE_NAME_SIZE: usize>(
    &self,
    pool: &'pl DiagnosticPool<'pl, NODE_NAME_SIZE>,
  ) -> DiagnosticReference<'pl, NODE_NAME_SIZE> {
    DiagnosticReference {
      index: self.index,
      generation: self.generation,
      pool,
    }
  }
}

// impl<'pool, const NODE_NAME_SIZE: usize> Debug for DiagnosticReference<'pool, NODE_NAME_SIZE> {
//   fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//     f.write_fmt(format_args!("DiagnosticReference[{} @ {}]", self.index, self.generation))
//   }
// }

impl<'pool, const NODE_NAME_SIZE: usize> Eq for DiagnosticReference<'pool, NODE_NAME_SIZE> {}
impl<'pool, const NODE_NAME_SIZE: usize> PartialEq for DiagnosticReference<'pool, NODE_NAME_SIZE> {
  fn eq(&self, other: &Self) -> bool { self.dereference() == other.dereference() }
}
