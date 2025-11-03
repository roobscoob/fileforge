use fileforge_lib::{
  binary_reader::{
    readable::{IntoReadable, NoneArgument},
    BinaryReader,
  },
  stream::{ReadableStream, RestorableStream},
};

use crate::sead::yaz0::{
  header::{readable::Yaz0HeaderReadError, Yaz0Header},
  parser::Yaz0Parser,
  readable::sealed::Sealed,
  state::Yaz0State,
  store::{NoSnapshots, Snapshots},
  MaybeSnapshotStore, Yaz0Stream,
};

mod sealed {
  pub trait Sealed {}
  // Only your internal types implement Sealed:
}

pub trait Yaz0StreamReadArgument<S: ReadableStream>: sealed::Sealed {
  type StoreType: MaybeSnapshotStore<S>;
}

pub struct Immutable;
pub struct Mutable;

impl Sealed for Immutable {}
impl Sealed for Mutable {}

impl<S: ReadableStream> Yaz0StreamReadArgument<S> for Immutable {
  type StoreType = NoSnapshots;
}

impl<S: RestorableStream> Yaz0StreamReadArgument<S> for Mutable {
  type StoreType = Snapshots<S::Snapshot, S>;
}

impl<'pool, S: ReadableStream<Type = u8>, A: Yaz0StreamReadArgument<Yaz0Parser<S>>> IntoReadable<'pool, S> for Yaz0Stream<S, A> {
  type Argument = A;
  type Error = Yaz0HeaderReadError<'pool, S::ReadError>;

  async fn read(mut reader: BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    let header: Yaz0Header = reader.read().await?;

    Ok(Yaz0Stream {
      state: Yaz0State::empty(),
      stream: Yaz0Parser::new(reader.into_stream(), header.decompressed_size()),
      header,
      store: A::StoreType::default(),
    })
  }
}

impl NoneArgument for Mutable {
  fn none() -> Self {
    Mutable
  }
}
