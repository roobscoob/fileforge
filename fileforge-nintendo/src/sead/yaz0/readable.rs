use std::{convert::Infallible, future::Future};

use fileforge_lib::{
  binary_reader::{
    error::static_subfork::StaticSubforkError,
    readable::{IntoReadable, NoneArgument, Readable},
    view::{View, ViewMutateError},
    BinaryReader,
  },
  error::FileforgeError,
  stream::{MutableStream, ReadableStream, RestorableStream, StaticPartitionableStream},
};

use crate::sead::yaz0::{
  header::{mutable::Yaz0HeaderMutator, Yaz0Header, YAZ0_HEADER_SIZE},
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

pub trait HeaderView<'pool, S: ReadableStream<Type = u8>>: Sized {
  type CreateError: FileforgeError;
  type OtherStream: ReadableStream<Type = u8>;

  fn create(reader: BinaryReader<'pool, S>) -> impl Future<Output = Result<(Self, Self::OtherStream), Self::CreateError>>;

  fn value(&self) -> &Yaz0Header;
}

impl<'pool, S: ReadableStream<Type = u8>> HeaderView<'pool, S> for Yaz0Header {
  type CreateError = <Yaz0Header as Readable<'pool, S>>::Error;
  type OtherStream = S;

  async fn create(mut reader: BinaryReader<'pool, S>) -> Result<(Self, S), Self::CreateError> {
    Ok((reader.read::<Yaz0Header>().await?, reader.into_stream()))
  }

  fn value(&self) -> &Yaz0Header {
    self
  }
}

pub trait MutHeaderView<'pool, S1: ReadableStream<Type = u8>, S2: MutableStream<Type = u8> + RestorableStream>: HeaderView<'pool, S1> {
  fn mutate<'l>(&'l mut self) -> impl Future<Output = Result<Yaz0HeaderMutator<'pool, 'l, S2, 0>, ViewMutateError<'pool, S2, Yaz0Header>>>
  where
    'pool: 'l,
    S2: 'l;
}

pub enum HeaderViewError<'pool, S1: StaticPartitionableStream<YAZ0_HEADER_SIZE, Type = u8>, S2: RestorableStream<Type = u8>> {
  Subfork(StaticSubforkError<'pool, YAZ0_HEADER_SIZE, S1>),
  Into(<View<'pool, S2, Yaz0Header> as IntoReadable<'pool, S2>>::Error),
}

impl<'pool, S1: StaticPartitionableStream<YAZ0_HEADER_SIZE, Type = u8>, S2: RestorableStream<Type = u8>> FileforgeError for HeaderViewError<'pool, S1, S2> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: fileforge_lib::diagnostic::pool::DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'poolx> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'poolx, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    todo!()
  }
}

impl<'pool, R: RestorableStream<Type = u8>, S: StaticPartitionableStream<YAZ0_HEADER_SIZE, Type = u8, PartitionLeft = R>> HeaderView<'pool, S> for View<'pool, S::PartitionLeft, Yaz0Header> {
  type CreateError = HeaderViewError<'pool, S, S::PartitionLeft>;
  type OtherStream = S::PartitionRight;

  async fn create(reader: BinaryReader<'pool, S>) -> Result<(Self, S::PartitionRight), Self::CreateError> {
    let (l, r) = reader.partition(Some("Yaz0 Header")).await.map_err(|e| HeaderViewError::Subfork(e))?;

    Ok((l.into::<View<'pool, _, Yaz0Header>>().await.map_err(|e| HeaderViewError::Into(e))?, r.into_stream()))
  }

  fn value(&self) -> &Yaz0Header {
    &**self
  }
}

impl<'pool, P: MutableStream<Type = u8> + RestorableStream, S: StaticPartitionableStream<YAZ0_HEADER_SIZE, Type = u8, PartitionLeft = P>> MutHeaderView<'pool, S, P> for View<'pool, P, Yaz0Header> {
  async fn mutate<'l>(&'l mut self) -> Result<Yaz0HeaderMutator<'pool, 'l, P, 0>, ViewMutateError<'pool, P, Yaz0Header>>
  where
    'pool: 'l,
  {
    self.mutate().await
  }
}

pub trait Yaz0StreamReadArgument<'pool, S1: ReadableStream<Type = u8>>: sealed::Sealed {
  type StoreType: MaybeSnapshotStore<<Self::HeaderView as HeaderView<'pool, S1>>::OtherStream>;
  type HeaderView: HeaderView<'pool, S1>;
}

pub struct Immutable;
pub struct Mutable;

impl Sealed for Immutable {}
impl Sealed for Mutable {}

impl<'pool, S1: ReadableStream<Type = u8>> Yaz0StreamReadArgument<'pool, S1> for Immutable {
  type StoreType = NoSnapshots;
  type HeaderView = Yaz0Header;
}

impl<'pool, S1: RestorableStream<Type = u8> + StaticPartitionableStream<YAZ0_HEADER_SIZE, PartitionLeft = S2, PartitionRight = S3>, S2: RestorableStream<Type = u8>, S3: RestorableStream<Type = u8>>
  Yaz0StreamReadArgument<'pool, S1> for Mutable
{
  type StoreType = Snapshots<S3::Snapshot, S3>;
  type HeaderView = View<'pool, S2, Yaz0Header>;
}

impl<'pool, S: ReadableStream<Type = u8>, A: Yaz0StreamReadArgument<'pool, S>> IntoReadable<'pool, S> for Yaz0Stream<'pool, S, A> {
  type Argument = A;
  type Error = <A::HeaderView as HeaderView<'pool, S>>::CreateError;

  async fn read(reader: BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    let (header, stream) = A::HeaderView::create(reader).await?;

    Ok(Yaz0Stream {
      state: Yaz0State::empty(),
      stream: Yaz0Parser::new(stream, header.value().decompressed_size()),
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

impl NoneArgument for Immutable {
  fn none() -> Self {
    Immutable
  }
}
