use crate::{error::FileforgeError, stream::MutableStream};

use super::BinaryReader;

pub trait Mutable<'pool, S: MutableStream<Type = u8>>: Sized {
  type Error: FileforgeError;
  type Mutator<'l>: 'l
  where
    'pool: 'l,
    Self: 'l,
    S: 'l;

  async fn mutate<'l>(reader: &'l mut BinaryReader<'pool, S>) -> Result<Self::Mutator<'l>, Self::Error>
  where
    Self: 'l;
}
