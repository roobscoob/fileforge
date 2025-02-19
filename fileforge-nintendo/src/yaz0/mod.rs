pub mod header;
pub mod error;
pub mod decode;
pub mod chunk;

use std::future::Future;

use chunk::{Yaz0Chunk, Yaz0Group};
use error::{construct::Yaz0ConstructionError, decode::Yaz0DecodeError, decompress::Yaz0DecompressError};
use fileforge_lib::{reader::{readable::{error::readable::ReadableError, Readable}, Reader}, stream::{error::stream_read::StreamReadError, ReadableStream}};

const MAX_YAZ0_LOOK_BACK_SIZE: usize = 0xFFF + 1;

pub struct Yaz0Stream<'pool, 'l, const NODE_NAME_SIZE: usize, UnderlyingStream: ReadableStream<NODE_NAME_SIZE>> {
    uncompressed_data_size: u32,
    requested_data_alignment: u32,

    // state
    stream: &'l mut Reader<'pool, NODE_NAME_SIZE, UnderlyingStream>,
    peeked_chunk: Option<Yaz0Chunk>,
    working_group: heapless::Deque<Yaz0Chunk, 8>,
    working_group_index: u32,
    decompressed_offset: u32,
    decoded_offset: u32,
    dictionary: heapless::HistoryBuffer<u8, MAX_YAZ0_LOOK_BACK_SIZE>,
}

impl<'pool, 'l, const NODE_NAME_SIZE: usize, UnderlyingStream: ReadableStream<NODE_NAME_SIZE>> Yaz0Stream<'pool, 'l, NODE_NAME_SIZE, UnderlyingStream> {
    pub fn requested_data_alignment(&self) -> u32 {
        self.requested_data_alignment
    }

    pub (self) async fn next_chunk(&mut self) -> Result<Option<Yaz0Chunk>, Yaz0DecodeError<'pool, NODE_NAME_SIZE, UnderlyingStream::ReadError>> {
        if let Some(next_chunk) = self.working_group.pop_back() {
            return Ok(Some(next_chunk))
        }

        if self.decoded_offset >= self.uncompressed_data_size {
            return Ok(None);
        }

        let group = self.stream.read_with::<Yaz0Group>((self.uncompressed_data_size - self.decoded_offset, self.working_group_index)).await?;

        self.working_group = group.chunks;

        if let Some(next_chunk) = self.working_group.pop_back() {
            return Ok(Some(next_chunk))
        }

        Ok(None)

    }
}

impl<'pool, 'l, const NODE_NAME_SIZE: usize, UnderlyingStream: ReadableStream<NODE_NAME_SIZE>> Readable<'pool, 'l, NODE_NAME_SIZE> for Yaz0Stream<'pool, 'l, NODE_NAME_SIZE, UnderlyingStream> {
    type Argument = ();
    type Error<S: ReadableStream<NODE_NAME_SIZE> + 'l> = Yaz0ConstructionError;

    async fn read<S: ReadableStream<NODE_NAME_SIZE>>(reader: &'l mut Reader<'pool, NODE_NAME_SIZE, S>, argument: Self::Argument) -> Result<Self, ReadableError<'pool, NODE_NAME_SIZE, Self::Error<S>, S::ReadError>> {
        
    }
}

impl<'pool, 'l, const NODE_NAME_SIZE: usize, UnderlyingStream: ReadableStream<NODE_NAME_SIZE>> ReadableStream<NODE_NAME_SIZE> for Yaz0Stream<'pool, 'l, NODE_NAME_SIZE, UnderlyingStream> {
    type ReadError = Yaz0DecompressError<'pool, NODE_NAME_SIZE, UnderlyingStream::ReadError>;

    fn len(&self) -> Option<u64> { Some(self.uncompressed_data_size as u64) } 
    fn offset(&self) -> u64 { self.decompressed_offset as u64 }

    async fn read<const SIZE: usize, V, R: Future<Output = V>>(&mut self, reader: impl FnOnce(&[u8; SIZE]) -> R) -> Result<V, StreamReadError<NODE_NAME_SIZE, Self::ReadError>> {
        
    }
}