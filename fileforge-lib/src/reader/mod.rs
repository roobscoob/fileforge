use core::{any::type_name, future::{ready, Future}};

use diagnostic_store::{DiagnosticKind, DiagnosticStore};
use endianness::Endianness;
use error::{exhausted::ReaderExhaustedError, get_primitive::GetPrimitiveError};
use primitive::Primitive;
use readable::{error::readable::ReadableError, NoneArgument, Readable};

use crate::{
  diagnostic::{node::{name::DiagnosticNodeName, reference::DiagnosticReference}, value::DiagnosticValue},
  provider::{hint::ReadHint, Provider},
  stream::{
    builtin::provider::ProviderStream,
    error::{stream_read::StreamReadError, user_read::UserReadError},
    ReadableStream,
  },
};

pub mod diagnostic_store;
pub mod endianness;
pub mod error;
pub mod primitive;
pub mod readable;
pub mod subfork;

pub struct Reader<'pool, const NODE_NAME_SIZE: usize, S: ReadableStream<NODE_NAME_SIZE>> {
  stream: S,
  endianness: Endianness,
  diagnostics: DiagnosticStore<'pool, NODE_NAME_SIZE>,
}

impl<'pool, const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> Reader<'pool, NODE_NAME_SIZE, ProviderStream<NODE_NAME_SIZE, P>>
where
  P::ReadError: UserReadError<NODE_NAME_SIZE>,
{
  pub fn new_from_provider(provider: P, endianness: Endianness, hint: ReadHint) -> Self {
    Self {
      diagnostics: DiagnosticStore::new(),
      endianness,
      stream: ProviderStream::new(provider, hint),
    }
  }

  pub fn set_hint(&mut self, hint: ReadHint) { self.stream.set_read_hint(hint); }
}

impl<'pool, const NODE_NAME_SIZE: usize, S: ReadableStream<NODE_NAME_SIZE>> Reader<'pool, NODE_NAME_SIZE, S> {
  pub fn new(stream: S, endianness: Endianness) -> Self {
    Self {
      diagnostics: DiagnosticStore::new(),
      endianness,
      stream,
    }
  }

  #[inline]
  pub fn set_endianness(&mut self, endianness: Endianness) { self.endianness = endianness; }

  #[inline]
  pub fn set_diagnostic(&mut self, kind: DiagnosticKind, diagnostic: Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>) { self.diagnostics.set(kind, diagnostic); }

  #[inline]
  pub fn borrow_fork<'a>(&'a mut self) -> Reader<'pool, NODE_NAME_SIZE, &'a mut S> {
    Reader {
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

  pub fn create_physical_diagnostic(&self, offset: i64, length: Option<u64>, name: impl Into<DiagnosticNodeName<NODE_NAME_SIZE>>) -> Option<DiagnosticReference<'pool, NODE_NAME_SIZE>> {
    Some(self.diagnostics.get(DiagnosticKind::Reader)?.create_physical_child(((self.stream.offset() as i128) + offset as i128).try_into().unwrap(), length, name))
  }

  pub fn read<'a, P: Readable<'pool, 'a, NODE_NAME_SIZE> + 'a>(&'a mut self) -> impl Future<Output = Result<P, ReadableError<'pool, NODE_NAME_SIZE, <P as Readable<'pool, 'a, NODE_NAME_SIZE>>::Error<S>, S::ReadError>>> + 'a where P::Argument: NoneArgument {
    P::read(self, P::Argument::none())
  }

  pub fn read_with<'a, P: Readable<'pool, 'a, NODE_NAME_SIZE> + 'a>(&'a mut self, argument: P::Argument) -> impl Future<Output = Result<P, ReadableError<'pool, NODE_NAME_SIZE, <P as Readable<'pool, 'a, NODE_NAME_SIZE>>::Error<S>, S::ReadError>>> + 'a {
    P::read(self, argument)
  }
}

pub trait PrimitiveReader<'pool, const NODE_NAME_SIZE: usize, const SIZE: usize, S: ReadableStream<NODE_NAME_SIZE>> {
  fn get<P: Primitive<SIZE>>(&mut self) -> impl Future<Output = Result<P, GetPrimitiveError<'pool, NODE_NAME_SIZE, <S as ReadableStream<NODE_NAME_SIZE>>::ReadError>>>;
}

impl<'pool, const NODE_NAME_SIZE: usize, S: ReadableStream<NODE_NAME_SIZE>, const SIZE: usize> PrimitiveReader<'pool, NODE_NAME_SIZE, SIZE, S> for Reader<'pool, NODE_NAME_SIZE, S> {
  async fn get<'a, P: Primitive<SIZE>>(&'a mut self) -> Result<P, GetPrimitiveError<'pool, NODE_NAME_SIZE, <S as ReadableStream<NODE_NAME_SIZE>>::ReadError>> {
    self.stream.read(|data: &[u8; SIZE]| ready(P::read(data, self.endianness))).await.map_err(|e| {
      let typename = type_name::<P>();
      match e {
        StreamReadError::StreamExhausted(e) => GetPrimitiveError::ReaderExhausted(typename, ReaderExhaustedError {
          container: self.diagnostics.get(DiagnosticKind::Reader),
          read_length: DiagnosticValue(SIZE as u64, None),
          read_offset: e.read_offset,
          stream_length: DiagnosticValue(e.stream_length, self.diagnostics.get(DiagnosticKind::ReaderLength)),
        }),
        StreamReadError::User(u) => GetPrimitiveError::User(typename, u),
      }
    })
  }
}
