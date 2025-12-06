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

impl<UnderlyingStream: ReadableStream<Type = u8>> SfatTable<UnderlyingStream> {
  pub async fn into_entry(self, name: &[u8]) -> Result<Option<SfatEntry>, _> {
    let name_hash = {
      let hasher = SfntHasher::new(self.hash_multiplier, self.hash_mode);
      hasher.write(name);
      hasher.get_hash()
    };

    let matching_entries = self
      .reader
      .read_with(Contiguous::of::<SfatEntry>().with_item_count(self.header.file_count))
      .await
      .unwrap()
      .filter(async |it| it.filename_hash == name_hash)
      .collect::<heapless::Vec<SfatEntry, 255>>()
      .await?;
  }
}
