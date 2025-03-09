use fileforge_lib::stream::ReadableStream;

pub mod readable;
pub mod header;

pub struct Yaz0Stream<UnderlyingStream: ReadableStream> {
    stream: UnderlyingStream,
}