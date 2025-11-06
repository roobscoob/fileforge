use core::convert::Infallible;

use fileforge::stream::ReadableStream;

use crate::sead::sarc::structures::sfat::entry::SfatEntry;

pub mod entry;
pub mod header;

pub struct SfatStream<UnderlyingStream: ReadableStream<Type = u8>> {
  stream: UnderlyingStream,
}

impl<Underlying: ReadableStream<Type = u8>> ReadableStream for SfatStream<Underlying> {
  type Type = SfatEntry;

  type ReadError = ();
  type SkipError = Infallible;

  fn offset(&self) -> u64 {
    self.stream.offset() / 0x10
  }
}
