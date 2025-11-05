use crate::{error::FileforgeError, stream::ResizableStream};

use super::BinaryReader;

pub trait Writable<'pool: 'l, 'l, S: ResizableStream<Type = u8> + 'l>: Sized {
  type Error: FileforgeError;

  async fn overwrite_into(&self, reader: &'l mut BinaryReader<'pool, S>) -> Result<(), Self::Error>;
}
