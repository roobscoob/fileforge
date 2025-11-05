use core::{any::type_name, future::ready};

use diagnostic_store::{DiagnosticKind, DiagnosticStore};
use endianness::Endianness;
use error::{
  exhausted::ReaderExhaustedError,
  get_primitive::GetPrimitiveError,
  rewind::RewindError,
  seek_out_of_bounds::{SeekOffset, SeekOutOfBounds},
  set_primitive::SetPrimitiveError,
  skip::SkipError,
};
use mutable::Mutable;
use primitive::Primitive;
use readable::{NoneArgument, Readable};
use writable::Writable;

use crate::{
  binary_reader::{readable::IntoReadable, snapshot::BinaryReaderSnapshot},
  diagnostic::{node::reference::DiagnosticReference, value::DiagnosticValue},
  provider::{hint::ReadHint, Provider},
  stream::{
    builtin::provider::ProviderStream,
    error::{
      stream_mutate::StreamMutateError, stream_read::StreamReadError, stream_restore::StreamRestoreError, stream_rewind::StreamRewindError, stream_seek_out_of_bounds::StreamSeekOutOfBoundsError,
      stream_skip::StreamSkipError, user_read::UserReadError,
    },
    MutableStream, ReadableStream, ResizableStream, RestorableStream, RewindableStream,
  },
};

pub mod diagnostic_store;
pub mod endianness;
pub mod error;
pub mod mutable;
pub mod primitive;
pub mod readable;
pub mod snapshot;
pub mod subfork;
pub mod view;
pub mod writable;

pub struct BinaryReader<'pool, S: ReadableStream<Type = u8>> {
  stream: S,
  endianness: Endianness,
  diagnostics: DiagnosticStore<'pool>,
}

impl<'pool, P: Provider<Type = u8>> BinaryReader<'pool, ProviderStream<P>>
where
  P::ReadError: UserReadError,
{
  pub fn new_from_provider(provider: P, endianness: Endianness, hint: ReadHint) -> Self {
    Self {
      diagnostics: DiagnosticStore::new(),
      endianness,
      stream: ProviderStream::new(provider, hint),
    }
  }

  pub fn set_hint(&mut self, hint: ReadHint) {
    self.stream.set_read_hint(hint);
  }
}

impl<'pool, S: ReadableStream<Type = u8>> BinaryReader<'pool, S> {
  pub fn new(stream: S, endianness: Endianness) -> Self {
    Self {
      diagnostics: DiagnosticStore::new(),
      endianness,
      stream,
    }
  }

  #[inline]
  pub fn set_endianness(&mut self, endianness: Endianness) {
    self.endianness = endianness;
  }

  #[inline]
  pub fn get_endianness(&mut self) -> Endianness {
    self.endianness
  }

  #[inline]
  pub fn set_diagnostic(&mut self, kind: DiagnosticKind, diagnostic: Option<DiagnosticReference<'pool>>) {
    self.diagnostics.set(kind, diagnostic);
  }

  #[inline]
  pub fn borrow_fork<'a>(&'a mut self) -> BinaryReader<'pool, &'a mut S> {
    BinaryReader {
      diagnostics: DiagnosticStore::new(),
      endianness: self.endianness,
      stream: &mut self.stream,
    }
  }

  pub fn fork(&self) -> Self
  where
    S: Clone,
  {
    Self {
      diagnostics: DiagnosticStore::new(),
      endianness: self.endianness,
      stream: self.stream.clone(),
    }
  }

  pub fn offset(&self) -> u64 {
    self.stream.offset()
  }

  pub fn create_physical_diagnostic(&self, offset: i64, length: Option<u64>, name: &str) -> Option<DiagnosticReference<'pool>> {
    Some(
      self
        .diagnostics
        .get(DiagnosticKind::Reader)?
        .create_physical_child(((self.stream.offset() as i128) + offset as i128).try_into().unwrap(), length, name),
    )
  }

  pub async fn read<P: Readable<'pool, S>>(&mut self) -> Result<P, P::Error>
  where
    P::Argument: NoneArgument,
  {
    P::read(self, P::Argument::none()).await
  }

  pub async fn into<P: IntoReadable<'pool, S>>(self) -> Result<P, P::Error>
  where
    P::Argument: NoneArgument,
  {
    P::read(self, P::Argument::none()).await
  }

  pub async fn read_with<P: Readable<'pool, S>>(&mut self, argument: P::Argument) -> Result<P, P::Error> {
    P::read(self, argument).await
  }

  pub async fn into_with<P: IntoReadable<'pool, S>>(self, argument: P::Argument) -> Result<P, P::Error> {
    P::read(self, argument).await
  }

  pub async fn skip(&mut self, size: u64) -> Result<(), SkipError<'pool, S::SkipError>> {
    self.stream.skip(size).await.map_err(|e| match e {
      StreamSkipError::User(u) => SkipError::User(u),
      StreamSkipError::SeekPointOverflowed {
        stream_length,
        offset,
        seek_forwards_distance,
      } => SkipError::OutOfBounds(SeekOutOfBounds {
        seek_offset: SeekOffset::Overflowed {
          base_offset: offset,
          add: seek_forwards_distance,
        },
        provider_size: DiagnosticValue(stream_length, self.diagnostics.get(DiagnosticKind::ReaderLength)),
        container_dr: self.diagnostics.get(DiagnosticKind::Reader),
      }),
      StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError { stream_length, seek_point }) => SkipError::OutOfBounds(SeekOutOfBounds {
        seek_offset: SeekOffset::InBounds(seek_point),
        provider_size: DiagnosticValue(stream_length, self.diagnostics.get(DiagnosticKind::ReaderLength)),
        container_dr: self.diagnostics.get(DiagnosticKind::Reader),
      }),
    })
  }

  pub fn into_stream(self) -> S {
    self.stream
  }
}

impl<'pool, S: RewindableStream<Type = u8>> BinaryReader<'pool, S> {
  pub async fn rewind(&mut self, size: u64) -> Result<(), RewindError<'pool, S::RewindError>> {
    self.stream.rewind(size).await.map_err(|e| match e {
      StreamRewindError::User(u) => RewindError::User(u),
      StreamRewindError::SeekPointUnderflowed {
        stream_length,
        offset,
        seek_backwards_distance,
      } => RewindError::OutOfBounds(SeekOutOfBounds {
        seek_offset: SeekOffset::Underflow {
          base_offset: offset,
          subtract: seek_backwards_distance,
        },
        provider_size: DiagnosticValue(stream_length, self.diagnostics.get(DiagnosticKind::ReaderLength)),
        container_dr: self.diagnostics.get(DiagnosticKind::Reader),
      }),
    })
  }
}

pub trait PrimitiveReader<'pool, const SIZE: usize, S: ReadableStream<Type = u8>> {
  async fn get<P: Primitive<SIZE>>(&mut self) -> Result<P, GetPrimitiveError<'pool, S::ReadError>>;
}

impl<'pool, S: ReadableStream<Type = u8>, const SIZE: usize> PrimitiveReader<'pool, SIZE, S> for BinaryReader<'pool, S> {
  async fn get<'a, P: Primitive<SIZE>>(&'a mut self) -> Result<P, GetPrimitiveError<'pool, <S as ReadableStream>::ReadError>> {
    self.stream.read(|data: &[u8; SIZE]| ready(P::read(data, self.endianness))).await.map_err(|e| {
      let typename = type_name::<P>();
      match e {
        StreamReadError::StreamExhausted(e) => GetPrimitiveError::ReaderExhausted(
          typename,
          ReaderExhaustedError {
            container: self.diagnostics.get(DiagnosticKind::Reader),
            length: DiagnosticValue(SIZE as u64, None),
            offset: e.read_offset,
            stream_length: DiagnosticValue(e.stream_length, self.diagnostics.get(DiagnosticKind::ReaderLength)),
          },
        ),
        StreamReadError::User(u) => GetPrimitiveError::User(typename, u),
      }
    })
  }
}

pub trait PrimitiveWriter<'pool, const SIZE: usize, S: MutableStream<Type = u8>> {
  async fn set<P: Primitive<SIZE>>(&mut self, primitive: P) -> Result<(), SetPrimitiveError<'pool, S::MutateError>>;
}

impl<'pool, S: MutableStream<Type = u8>, const SIZE: usize> PrimitiveWriter<'pool, SIZE, S> for BinaryReader<'pool, S> {
  async fn set<P: Primitive<SIZE>>(&mut self, primitive: P) -> Result<(), SetPrimitiveError<'pool, S::MutateError>> {
    self.stream.mutate(|data: &mut [u8; SIZE]| ready(P::write(&primitive, data, self.endianness))).await.map_err(|e| {
      let typename = type_name::<P>();
      match e {
        StreamMutateError::StreamExhausted(e) => SetPrimitiveError::ReaderExhausted(
          typename,
          ReaderExhaustedError {
            container: self.diagnostics.get(DiagnosticKind::Reader),
            length: DiagnosticValue(SIZE as u64, None),
            offset: e.read_offset,
            stream_length: DiagnosticValue(e.stream_length, self.diagnostics.get(DiagnosticKind::ReaderLength)),
          },
        ),
        StreamMutateError::User(u) => SetPrimitiveError::User(typename, u),
      }
    })
  }
}

pub trait MutableMutator<'pool, S: MutableStream<Type = u8>> {
  async fn mutate<'l, M: Mutable<'pool, S> + 'l>(&'l mut self) -> Result<M::Mutator<'l>, M::Error>
  where
    'pool: 'l,
    S: 'l;
}

impl<'pool, S: MutableStream<Type = u8>> MutableMutator<'pool, S> for BinaryReader<'pool, S> {
  async fn mutate<'l, M: Mutable<'pool, S> + 'l>(&'l mut self) -> Result<M::Mutator<'l>, M::Error> {
    M::mutate(self).await
  }
}

impl<'pool, S: ResizableStream<Type = u8>> BinaryReader<'pool, S> {
  pub async fn overwrite<'l, W: Writable<'pool, 'l, S>>(&'l mut self, writable: &'l W) -> Result<(), W::Error> {
    W::overwrite_into(writable, self).await
  }
}

impl<'pool, S: RestorableStream<Type = u8>> BinaryReader<'pool, S> {
  pub fn snapshot(&self) -> BinaryReaderSnapshot<'pool, S> {
    BinaryReaderSnapshot {
      snapshot: self.stream.snapshot(),
      endianness: self.endianness,
      diagnostics: self.diagnostics.clone(),
    }
  }

  pub async fn restore(&mut self, snapshot: BinaryReaderSnapshot<'pool, S>) -> Result<(), StreamRestoreError<S::RestoreError>> {
    self.stream.restore(snapshot.snapshot).await?;
    self.endianness = snapshot.endianness;
    self.diagnostics = snapshot.diagnostics;

    Ok(())
  }
}
