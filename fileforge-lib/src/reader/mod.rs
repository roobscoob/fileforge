use core::future::{ready, Future};

use diagnostic_store::{DiagnosticKind, DiagnosticStore};
use endianness::Endianness;
use error::{exhausted::ReaderExhaustedError, get_primitive::GetPrimitiveError};
use primitive::Primitive;

use crate::{
  diagnostic::node::reference::DiagnosticReference,
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

  pub fn set_endianness(&mut self, endianness: Endianness) { self.endianness = endianness; }

  pub fn set_diagnostic(&mut self, kind: DiagnosticKind, diagnostic: DiagnosticReference<'pool, NODE_NAME_SIZE>) { self.diagnostics.set(kind, diagnostic); }

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
}

pub trait PrimitiveReader<const NODE_NAME_SIZE: usize, const SIZE: usize, S: ReadableStream<NODE_NAME_SIZE>> {
  fn get<P: Primitive<SIZE>>(&mut self) -> impl Future<Output = Result<P, GetPrimitiveError<NODE_NAME_SIZE, <S as ReadableStream<NODE_NAME_SIZE>>::ReadError>>>;
}

impl<'pool, const NODE_NAME_SIZE: usize, S: ReadableStream<NODE_NAME_SIZE>, const SIZE: usize> PrimitiveReader<NODE_NAME_SIZE, SIZE, S> for Reader<'pool, NODE_NAME_SIZE, S> {
  async fn get<P: Primitive<SIZE>>(&mut self) -> Result<P, GetPrimitiveError<NODE_NAME_SIZE, <S as ReadableStream<NODE_NAME_SIZE>>::ReadError>> {
    self.stream.read(|data: &[u8; SIZE]| ready(P::read(data, self.endianness))).await.map_err(|e| match e {
      StreamReadError::StreamExhausted(e) => GetPrimitiveError::ReaderExhausted(ReaderExhaustedError {}),
      StreamReadError::User(u) => GetPrimitiveError::User(u),
    })
  }
}
