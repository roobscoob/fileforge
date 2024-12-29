use crate::{
  error::FileforgeError,
  provider::{
    error::{out_of_bounds::OutOfBoundsError, provider_read::ProviderReadError},
    hint::ReadHint,
    Provider,
  },
  stream::{
    error::{stream_exhausted::StreamExhaustedError, stream_read::StreamReadError, stream_skip::StreamSkipError, user_read::UserReadError},
    ReadableStream, SeekableStream,
  },
};

pub struct ProviderStream<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> {
  provider: P,
  hint: ReadHint,
  offset: u64,
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> ProviderStream<NODE_NAME_SIZE, P> {
  pub fn new(provider: P, hint: ReadHint) -> Self { Self { provider, hint, offset: 0 } }
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> ReadableStream<u8, NODE_NAME_SIZE> for ProviderStream<NODE_NAME_SIZE, P>
where
  P::ReadError: UserReadError<NODE_NAME_SIZE>,
{
  type ReadError = P::ReadError;

  fn len(&self) -> Option<u64> { Some(self.provider.len()) }

  async fn read<const SIZE: usize, V, R: core::future::Future<Output = V>>(&mut self, reader: impl FnOnce(&[u8; SIZE]) -> R) -> Result<V, StreamReadError<NODE_NAME_SIZE, Self::ReadError>> {
    return match self.provider.read(self.offset, self.hint, reader).await {
      Ok(v) => {
        self.offset += SIZE as u64;

        Ok(v)
      }

      Err(ProviderReadError::User(u)) => Err(StreamReadError::User(u)),

      Err(ProviderReadError::OutOfBounds(oob)) => Err(StreamReadError::StreamExhausted(Option::<StreamExhaustedError>::from(oob).expect("Read length should be non-None"))),
    };
  }
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> SeekableStream<u8, NODE_NAME_SIZE> for ProviderStream<NODE_NAME_SIZE, P> {
  type SeekError = core::convert::Infallible;

  async fn seek(&mut self, offset: u64) -> Result<(), crate::stream::error::stream_seek::StreamSeekError<NODE_NAME_SIZE, Self::SeekError>> { self.offset = offset; }
}
