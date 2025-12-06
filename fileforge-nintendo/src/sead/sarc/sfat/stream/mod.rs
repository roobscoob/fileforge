use fileforge::{binary_reader::BinaryReader, stream::ReadableStream};

pub mod readable;

pub struct SfatStream<'pool, UnderlyingStream: ReadableStream<Type = u8>> {
  stream: BinaryReader<'pool, UnderlyingStream>,
}
