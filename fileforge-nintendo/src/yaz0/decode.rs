use std::future::ready;

use fileforge_lib::stream::{error::stream_read::StreamReadError, ReadableStream};

use super::{chunk::Yaz0Chunk, error::decode::Yaz0DecodeError, Yaz0Stream};


// impl<const NODE_NAME_SIZE: usize, UnderlyingStream: ReadableStream<NODE_NAME_SIZE>> Yaz0Decoder<NODE_NAME_SIZE, UnderlyingStream> {
    

//     async fn saturate_working_group(&mut self) -> Result<u8, Yaz0DecodeError> {
//         let header = self.stream.read(|header: &[u8; 1]| ready(header[0])).await?;

//         for index in 0..7 {
//             let mask = 0b1 << index;
//             let bit = header & mask != 0;

//             let chunk = if bit {
//                 Yaz0Chunk::Literal(self.stream.read(|header: &[u8; 1]| ready(header[0])).await?)
//             } else {
//                 let heading = self.stream.read(|header: &[u8; 2]| ready(*header)).await?;
                
//                 let distance = (((heading[0] & 0xF) as u16) << 8) + heading[1] as u16;
//                 let length = if heading[0] & 0xF0 == 0 {
//                     self.stream.read(|header: &[u8; 1]| ready(header[0] as u16)).await? + 0x12
//                 } else {
//                     (heading[0] as u16 & 0xF0 >> 4) + 0x2
//                 };

//                 Yaz0Chunk::Backreference { distance, length }
//             };

//             self.working_group.push_front(chunk)
//         }
//     }
// }

pub struct Yaz0Decoder<'pool, 'l, const NODE_NAME_SIZE: usize, UnderlyingStream: ReadableStream<NODE_NAME_SIZE>> {
    decoder: &'l mut Yaz0Stream<'pool, 'l, NODE_NAME_SIZE, UnderlyingStream>,
    goal: u64,
}

impl<'pool, 'l, const NODE_NAME_SIZE: usize, UnderlyingStream: ReadableStream<NODE_NAME_SIZE>> Yaz0Decoder<'pool, 'l, NODE_NAME_SIZE, UnderlyingStream> {
    fn new(decoder: &'l mut Yaz0Stream<'pool, 'l, NODE_NAME_SIZE, UnderlyingStream>, goal: u64) -> Yaz0Decoder<'pool, 'l, NODE_NAME_SIZE, UnderlyingStream> {
        Yaz0Decoder { decoder, goal }
    }    

    // async fn next(&mut self) -> Option<Yaz0Chunk> {
    //     if self.goal == 0 {
    //         return None;
    //     }

    //     let next_chunk = if let Some(next_chunk) = self.decoder.peeked_chunk {
    //         next_chunk
    //     } else { 
    //         self.decoder.next_chunk()
    //     }

    //     if let Some(next_chunk) = next_chunk {
    //         let (current, remaining) = peeked_chunk.split(self.goal);

    //         self.goal -= current.len();
    //         self.decoder.peeked_chunk = remaining;

    //         return Some(current);
    //     }
    // }
}