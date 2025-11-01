use crate::{error::FileforgeError, stream::MutableStream};

use super::BinaryReader;

pub trait Writable<'pool: 'l, 'l, S: MutableStream<Type = u8> + 'l>: Sized {
  type Error: FileforgeError;

  async fn overwrite_into(&self, reader: &'l mut BinaryReader<'pool, S>) -> Result<(), Self::Error>;
}
