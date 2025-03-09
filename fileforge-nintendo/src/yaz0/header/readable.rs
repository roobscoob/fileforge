use fileforge_lib::{error::FileforgeError, reader::{error::get_primitive::GetPrimitiveError, readable::Readable, PrimitiveReader, Reader}, stream::{error::user_read::UserReadError, ReadableStream}};
use fileforge_std::magic::{error::error::MagicError, Magic};

use super::Yaz0Header;

pub const YAZ0_MAGIC: Magic<4> = Magic::from_byte_ref(b"Yaz0");

impl<'pool: 'l, 'l, S: ReadableStream + 'l> Readable<'pool, 'l, S> for Yaz0Header {
    type Error = Yaz0HeaderReadError<'pool, S::ReadError>;
    type Argument = ();

    async fn read(reader: &'l mut Reader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
        reader.read_with::<Magic<4>>(YAZ0_MAGIC).await.map_err(|e| Yaz0HeaderReadError::Magic(e))?;

        Ok(Yaz0Header {
            decompressed_size: reader.get().await.map_err(|e| Yaz0HeaderReadError::TotalSize(e))?,
            data_alignment: reader.get().await.map_err(|e| Yaz0HeaderReadError::Alignment(e))?,
            unused: reader.get().await.map_err(|e| Yaz0HeaderReadError::Unused(e))?
        })
    }
}

pub enum Yaz0HeaderReadError<'pool, U: UserReadError> {
    Magic(MagicError<'pool, 4, U>),
    TotalSize(GetPrimitiveError<'pool, U>),
    Alignment(GetPrimitiveError<'pool, U>),
    Unused(GetPrimitiveError<'pool, U>),
}

impl<'pool, U: UserReadError> FileforgeError for Yaz0HeaderReadError<'pool, U> {
    fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: fileforge_lib::diagnostic::pool::DiagnosticPoolProvider>(&self, provider: &'pool_ref P, callback: impl for<'tag, 'b, 'p2> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> ()) {
        unimplemented!()
    }
}