use crate::{error::FileforgeError, stream::MutableStream};

use super::Reader;

pub trait Writable<'pool: 'l, 'l, S: MutableStream + 'l>: Sized {
    type Error: FileforgeError;

    async fn overwrite_into(&self, reader: &'l mut Reader<'pool, S>) -> Result<(), Self::Error>;
}