pub mod entry;
pub mod hasher;

use fileforge::{binary_reader::BinaryReader, stream::ReadableStream};

use crate::sead::sarc::sfat::name_table::{entry::NameTableEntry, hasher::SfntHasher};

pub struct SarcNameTable<'pool, UnderlyingStream: ReadableStream<Type = u8>> {
  length: u32,
  hasher: SfntHasher,
  stream: BinaryReader<'pool, UnderlyingStream>,
}

impl<'pool, UnderlyingStream: ReadableStream<Type = u8>> SarcNameTable<'pool, UnderlyingStream> {
  pub fn next<'a>(&'a mut self) -> Option<NameTableEntry<'a, 'pool, UnderlyingStream>> {
    if self.stream.offset() >= self.length as u64 {
      None
    } else {
      Some(NameTableEntry {
        stream: &mut self.stream,
        hasher: self.hasher,
      })
    }
  }
}
