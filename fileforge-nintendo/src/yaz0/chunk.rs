use fileforge_lib::{reader::{readable::{error::readable::ReadableError, Readable}, PrimitiveReader, Reader}, stream::ReadableStream};

use super::error::decode::Yaz0DecodeError;

pub enum Yaz0Chunk {
    Literal(u8),
    BackreferenceShort { distance: u16, length: u8 },
    BackreferenceLong { distance: u16, length: u8 },
}

impl Yaz0Chunk {
    #[inline]
    pub fn split(self, requested_length: usize) -> (Yaz0Chunk, Option<Yaz0Chunk>) {
        if requested_length == 0 {
            return (Yaz0Chunk::BackreferenceShort { distance: 0, length: 0 }, Some(self))
        }

        match self {
            Self::Literal(_) => (self, None),
            Self::BackreferenceShort { distance, length } => {
                let additional_bytes = ((length as i32) + 2) - (requested_length as i32);

                if additional_bytes < 0 {
                    (self, None)
                } else {
                    (Self::BackreferenceShort { distance, length: (requested_length as u8) - 2 }, Some(Self::BackreferenceShort { distance, length: (additional_bytes as u8) - 2 }))
                }
            }
            Self::BackreferenceLong { distance, length } => {
                let additional_bytes = ((length as i32) + 12) - (requested_length as i32);

                if additional_bytes < 0 {
                    (self, None)
                } else {
                    (Self::BackreferenceLong { distance, length: (requested_length as u8) - 12 }, Some(Self::BackreferenceLong { distance, length: (additional_bytes as u8) - 12 }))
                }
            }
        }
    }

    #[inline]
    pub fn content_length(&self) -> usize {
        match self {
            Self::Literal(_) => 1,
            Self::BackreferenceShort { length, .. } => (*length as usize) + 2,
            Self::BackreferenceLong { length, .. } => (*length as usize) + 12,
        }
    }

    #[inline]
    pub fn chunk_length(&self) -> usize {
        match self {
            Self::Literal(_) => 1,
            Self::BackreferenceShort { .. } => 2,
            Self::BackreferenceLong { .. } => 3,
        }
    }
}

impl<'pool, 'l, const NODE_NAME_SIZE: usize> Readable<'pool, 'l, NODE_NAME_SIZE> for Yaz0Chunk {
    type Error<S: ReadableStream<NODE_NAME_SIZE> + 'l> = Yaz0DecodeError<'pool, NODE_NAME_SIZE, S::ReadError> where 'pool: 'l;
    type Argument = (u8, u8, u8, u32);

    async fn read<S: ReadableStream<NODE_NAME_SIZE>>(reader: &'l mut Reader<'pool, NODE_NAME_SIZE, S>, (chunk_index, group_header, byte_offset, group_index): (u8, u8, u8, u32)) -> Result<Self, ReadableError<'pool, NODE_NAME_SIZE, Self::Error<S>, S::ReadError>> {
        if (group_header & (1u8 << chunk_index)) != 0 {
            let byte = reader.get::<u8>().await
                .map_err(|e| ReadableError::User(Yaz0DecodeError::missing_inline_byte(e, reader, chunk_index, group_header, byte_offset, group_index)))?;

            return Ok(Yaz0Chunk::Literal(byte));
        }

        let assumed_short_backreference = reader.get::<[u8; 2]>().await
            .map_err(|e| ReadableError::User(Yaz0DecodeError::missing_assumed_short_backreference(e, reader, chunk_index, group_header, byte_offset, group_index)))?;

        let distance = 
            ((assumed_short_backreference[0] & 0x0F) << 8) as u16
          & assumed_short_backreference[1] as u16
          + 1;

        if assumed_short_backreference[0] & 0xF0 != 0 {
            Ok(Yaz0Chunk::BackreferenceShort { distance, length: (assumed_short_backreference[0] & 0xF0) >> 4 })
        } else {
            let length = reader.get::<u8>().await
                .map_err(|e| ReadableError::User(Yaz0DecodeError::missing_long_backreference(e, reader, chunk_index, group_header, byte_offset, group_index)))?;

            Ok(Yaz0Chunk::BackreferenceLong { distance, length })
        }
    }
}

pub struct Yaz0Group {
    pub chunks: heapless::Deque<Yaz0Chunk, 8>,
}

impl<'pool, 'l, const NODE_NAME_SIZE: usize> Readable<'pool, 'l, NODE_NAME_SIZE> for Yaz0Group {
    type Error<S: ReadableStream<NODE_NAME_SIZE> + 'l> = Yaz0DecodeError<'pool, NODE_NAME_SIZE, S::ReadError> where 'pool: 'l;
    type Argument = (u32, u32);

    async fn read<S: ReadableStream<NODE_NAME_SIZE>>(reader: &'l mut Reader<'pool, NODE_NAME_SIZE, S>, (mut remaining_bytes, group_index): (u32, u32)) -> Result<Self, ReadableError<'pool, NODE_NAME_SIZE, Self::Error<S>, S::ReadError>> {
        let header = reader.get::<u8>().await
            .map_err(|e| ReadableError::User(Yaz0DecodeError::missing_header(e, reader, group_index)))?;

        let mut byte_offset = 0;
        let mut chunks = heapless::Deque::new();

        for chunk_index in 0..8 {
            if remaining_bytes == 0 {
                break;
            }

            let chunk = reader.read_with::<Yaz0Chunk>((chunk_index, header, byte_offset, group_index)).await?;

            byte_offset += chunk.chunk_length() as u8;
            remaining_bytes -= chunk.content_length() as u32;

            chunks.push_front(chunk);
        };

        Ok(Yaz0Group { chunks })
    }
}