use fileforge_lib::{
  binary_reader::{snapshot::BinaryReaderSnapshot, BinaryReader},
  stream::{ReadableStream, RestorableStream},
};

use crate::sead::yaz0::{parser::Yaz0Parser, state::Yaz0State};

mod sealed {
  pub trait Sealed {}
}

pub trait MaybeSnapshotStore<S: ReadableStream<Type = u8>>: Default + Clone + sealed::Sealed {
  fn store_snapshot(&mut self, stream: &Yaz0Parser<S>, state: Yaz0State);
}

pub trait MaybeHeaderSnapshotStore<'pool, S: ReadableStream<Type = u8>>: Default + Clone + sealed::Sealed {
  fn store_header_snapshot(&mut self, stream: &BinaryReader<'pool, S>);
}

pub trait SnapshotStore<R: RestorableStream<Type = u8>>: MaybeSnapshotStore<R> {
  fn snapshot(&self) -> Option<&<Yaz0Parser<R> as RestorableStream>::Snapshot>;
  fn state(&self) -> Yaz0State;
}

pub trait HeaderSnapshotStore<'pool, R: RestorableStream<Type = u8>>: MaybeHeaderSnapshotStore<'pool, R> {
  fn snapshot(&self) -> Option<&BinaryReaderSnapshot<R>>;
}

#[derive(Default, Clone)]
pub struct NoSnapshots;
impl sealed::Sealed for NoSnapshots {}
impl<S: ReadableStream<Type = u8>> MaybeSnapshotStore<S> for NoSnapshots {
  fn store_snapshot(&mut self, _: &Yaz0Parser<S>, _: Yaz0State) {}
}
impl<'pool, S: ReadableStream<Type = u8>> MaybeHeaderSnapshotStore<'pool, S> for NoSnapshots {
  fn store_header_snapshot(&mut self, _: &BinaryReader<'pool, S>) {}
}

pub struct Snapshots<Sn: Clone, S: RestorableStream<Type = u8, Snapshot = Sn>> {
  snapshot: Option<<Yaz0Parser<S> as RestorableStream>::Snapshot>,
  state: Yaz0State,
}

impl<S: RestorableStream<Type = u8>> sealed::Sealed for Snapshots<S::Snapshot, S> {}
impl<S: RestorableStream<Type = u8>> Clone for Snapshots<S::Snapshot, S> {
  fn clone(&self) -> Self {
    Self {
      snapshot: self.snapshot.clone(),
      state: self.state.clone(),
    }
  }
}

impl<S: RestorableStream<Type = u8>> Default for Snapshots<S::Snapshot, S> {
  fn default() -> Self {
    Self {
      snapshot: None,
      state: Yaz0State::empty(),
    }
  }
}

impl<R: RestorableStream<Type = u8>> MaybeSnapshotStore<R> for Snapshots<R::Snapshot, R> {
  fn store_snapshot(&mut self, stream: &Yaz0Parser<R>, state: Yaz0State) {
    self.snapshot = Some(stream.snapshot());
    self.state = state;
  }
}

impl<R: RestorableStream<Type = u8>> SnapshotStore<R> for Snapshots<R::Snapshot, R> {
  fn snapshot(&self) -> Option<&<Yaz0Parser<R> as RestorableStream>::Snapshot> {
    self.snapshot.as_ref()
  }

  fn state(&self) -> Yaz0State {
    self.state.clone()
  }
}
