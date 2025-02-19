use std::future::Future;

use fileforge_lib::{reader::{diagnostic_store::DiagnosticKind, readable::{error::readable::ReadableError, Readable}, PrimitiveReader, Reader}, stream::ReadableStream};
use fileforge_std::magic::{error::MagicError, Magic};

pub struct Yaz0Header {
    uncompressed_data_size: u32,
    data_alignment: u32,
    unused: u32,
}

static YAZ0_MAGIC: Magic<4> = Magic::from_byte_ref(b"Yaz0");

impl<'pool, 'l, const NODE_NAME_SIZE: usize> Readable<'pool, 'l, NODE_NAME_SIZE> for Yaz0Header {
    type Error<S: ReadableStream<NODE_NAME_SIZE> + 'l> = MagicError<'pool, NODE_NAME_SIZE, 4> where 'pool: 'l;
    type Argument = ();

    async fn read<S: ReadableStream<NODE_NAME_SIZE>>(parent: &'l mut Reader<'pool, NODE_NAME_SIZE, S>, _: ()) -> Result<Self, ReadableError<'pool, NODE_NAME_SIZE, Self::Error<S>, S::ReadError>> {
        let diagnostic = parent.create_physical_diagnostic(0, Some(16), "Yaz0Header");        

        let mut reader = parent.borrow_fork();

        reader.set_diagnostic(DiagnosticKind::Reader, diagnostic);

        reader.read_with::<Magic<4>>(YAZ0_MAGIC).await?;
                
        Ok(Yaz0Header {
            uncompressed_data_size: reader.get().await?,
            data_alignment: reader.get().await?,
            unused: reader.get().await?,
        })
    }
}