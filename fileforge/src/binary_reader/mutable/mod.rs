use crate::{error::FileforgeError, stream::MutableStream};

use super::BinaryReader;

/// A value that can be changed in place, through a mutator that writes its fields back to the stream.
pub trait Mutable<'pool, S: MutableStream<Type = u8>>: Sized {
  type Error: FileforgeError;
  type Mutator<'l>: 'l
  where
    'pool: 'l,
    Self: 'l,
    S: 'l;

  /// Starts changing `value`, which was read from `reader`'s position. The mutator should update
  /// `value` as it writes each field, so that it keeps matching the stream.
  async fn mutate<'l>(value: &'l mut Self, reader: &'l mut BinaryReader<'pool, S>) -> Result<Self::Mutator<'l>, Self::Error>
  where
    Self: 'l;
}
