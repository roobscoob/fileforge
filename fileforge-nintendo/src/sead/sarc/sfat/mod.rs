use fileforge::{binary_reader::BinaryReader, stream::ReadableStream};

use crate::sead::sarc::sfat::{header::SfatHeader, name_table::hasher::SfntHasher};

pub mod entry;
pub mod header;
pub mod name_table;
pub mod stream;

pub struct SfatTable<'pool, UnderlyingStream: ReadableStream<Type = u8>> {
  hasher: SfntHasher,
  header: SfatHeader,
  reader: BinaryReader<'pool, UnderlyingStream>,
}

// Note:
// there's a weird logic hole.
// if there's only one file in the SfatTable
// then i cannot validate hash collisions
