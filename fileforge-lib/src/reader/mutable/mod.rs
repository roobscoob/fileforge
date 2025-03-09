use crate::{error::FileforgeError, stream::MutableStream};

use super::Reader;

pub trait Mutable<'pool, 'l, S: MutableStream>: Sized {
    type Error: FileforgeError;
    type Mutator: 'l;

    async fn mutate(reader: &'l mut Reader<'pool, S>) -> Result<Self::Mutator, Self::Error>;
}