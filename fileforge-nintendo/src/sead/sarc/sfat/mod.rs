use core::hash::Hasher;

use fileforge::{
  binary_reader::BinaryReader,
  stream::{DynamicPartitionableStream, ReadableStream},
};

use crate::sead::sarc::sfat::{
  entry::SfatEntry,
  header::SfatHeader,
  name_table::{
    hasher::{HashMode, SfntHasher},
    SarcNameTable,
  },
  stream::SfatStream,
};

pub mod entry;
pub mod header;
pub mod name_table;
pub mod stream;

pub struct SfatTable<'pool, UnderlyingStream: ReadableStream<Type = u8>> {
  hash_mode: HashMode,
  hash_multiplier: u32,
  header: SfatHeader,
  reader: BinaryReader<'pool, UnderlyingStream>,
}

// Note:
// there's a weird logic hole.
// if there's only one file in the SfatTable
// then i cannot validate hash collisions
