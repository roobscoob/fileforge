pub mod endianness;
pub mod error;

use core::future::{ready, Future};

use endianness::Endianness;
use error::read_bytes::ReadBytesError;

use crate::{
  diagnostic::node::reference::DiagnosticReference,
  provider::{hint::ReadHint, Provider},
  stream::{provider::ProviderStream, ReadableStream},
};

pub struct Reader<'pool, S: ReadableStream, const NODE_NAME_SIZE: usize> {
  stream: S,
  body_diagnostic: DiagnosticReference<'pool, NODE_NAME_SIZE>,
  length_diagnostic: Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>,
  endianness: Endianness,
}

impl<'pool, S: ReadableStream, const NODE_NAME_SIZE: usize> Reader<'pool, S, NODE_NAME_SIZE> {
  pub async fn read_bytes_async<const SIZE: usize, V, F: Future<Output = V>>(&mut self, reader: impl FnOnce(&[u8; SIZE]) -> F) -> Result<V, ReadBytesError<'pool, NODE_NAME_SIZE, S::ReadError>> {
    match self.stream.read(reader).await {
      Ok(v) => Ok(v),
      Err(e)
    }
  }

  pub async fn read_bytes<const SIZE: usize, V>(&mut self, reader: impl FnOnce(&[u8; SIZE]) -> V) -> Result<V, S::ReadError> { self.stream.read(|data| ready(reader(data))).await }

  pub async fn skip(&mut self, size: u64) -> Result<(), S::SkipError> { self.stream.skip(size).await }
}
