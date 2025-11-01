use crate::{error::FileforgeError, stream::MutableStream};

use super::BinaryReader;

pub trait Mutable<'pool, 'l, S: MutableStream<Type = u8>>: Sized {
  type Error: FileforgeError;
  type Mutator: 'l;

  async fn mutate(reader: &'l mut BinaryReader<'pool, S>) -> Result<Self::Mutator, Self::Error>;
}
