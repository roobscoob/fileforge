use fileforge_lib::stream::{ReadableStream, RestorableStream};

use crate::sead::yaz0::state::Yaz0State;

mod sealed {
  pub trait Sealed {}
}

pub trait MaybeSnapshotStore<S: ReadableStream>: Default + Clone + sealed::Sealed {
  fn store_snapshot(&mut self, stream: &S, state: Yaz0State);
}
pub trait SnapshotStore<R: RestorableStream>: MaybeSnapshotStore<R> {
  fn snapshot(&self) -> Option<&R::Snapshot>;
  fn state(&self) -> Yaz0State;
}

#[derive(Default, Clone)]
pub struct NoSnapshots;
impl sealed::Sealed for NoSnapshots {}
impl<S: ReadableStream> MaybeSnapshotStore<S> for NoSnapshots {
  fn store_snapshot(&mut self, _: &S, _: Yaz0State) {}
}

pub struct Snapshots<Sn: Clone, S: RestorableStream<Snapshot = Sn>> {
  snapshot: Option<S::Snapshot>,
  state: Yaz0State,
}

impl<S: RestorableStream> sealed::Sealed for Snapshots<S::Snapshot, S> {}
impl<S: RestorableStream> Clone for Snapshots<S::Snapshot, S> {
  fn clone(&self) -> Self {
    Self {
      snapshot: self.snapshot.clone(),
      state: self.state.clone(),
    }
  }
}

impl<S: RestorableStream> Default for Snapshots<S::Snapshot, S> {
  fn default() -> Self {
    Self {
      snapshot: None,
      state: Yaz0State::empty(),
    }
  }
}

impl<R: RestorableStream> MaybeSnapshotStore<R> for Snapshots<R::Snapshot, R> {
  fn store_snapshot(&mut self, stream: &R, state: Yaz0State) {
    self.snapshot = Some(stream.snapshot());
    self.state = state;
  }
}

impl<R: RestorableStream> SnapshotStore<R> for Snapshots<R::Snapshot, R> {
  fn snapshot(&self) -> Option<&<R as RestorableStream>::Snapshot> {
    self.snapshot.as_ref()
  }

  fn state(&self) -> Yaz0State {
    self.state.clone()
  }
}
