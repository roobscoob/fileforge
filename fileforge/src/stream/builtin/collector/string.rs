use crate::stream::{collectable::Collectable, ReadableStream, StreamReadError, SINGLE};

impl<S: ReadableStream<Type = char>> Collectable<S> for alloc::string::String {
  type Error = <S as ReadableStream>::ReadError;

  async fn collect(&mut self, stream: &mut S) -> Result<(), Self::Error> {
    loop {
      self.push(match stream.read(SINGLE).await {
        Ok(c) => c,
        Err(StreamReadError::StreamExhausted(_)) => break,
        Err(StreamReadError::User(u)) => return Err(u),
      });
    }

    Ok(())
  }
}
