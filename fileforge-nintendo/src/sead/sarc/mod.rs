pub mod header;
pub mod readable;
pub mod sfat;

use fileforge::{binary_reader::BinaryReader, stream::ReadableStream};

use crate::sead::sarc::{
  header::{SarcHeader, SARC_HEADER_SIZE},
  sfat::{
    entry::SFAT_ENTRY_SIZE,
    header::{SfatHeader, SFAT_HEADER_SIZE},
    name_table::header::SFNT_HEADER_SIZE,
  },
};

/// A SARC archive, read from a stream it owns.
///
/// Opening one reads the SARC, SFAT and SFNT headers; files are read when they're asked for.
pub struct Sarc<'pool, S: ReadableStream<Type = u8>> {
  reader: BinaryReader<'pool, S>,
  header: SarcHeader,
  sfat: SfatHeader,
}

impl<'pool, S: ReadableStream<Type = u8>> Sarc<'pool, S> {
  pub fn header(&self) -> &SarcHeader {
    &self.header
  }

  pub fn file_count(&self) -> u16 {
    self.sfat.file_count
  }

  pub fn hash_multiplier(&self) -> u32 {
    self.sfat.hash_multiplier
  }

  /// Where the SFAT's entries start, from the start of the archive.
  pub fn entries_offset(&self) -> u64 {
    (SARC_HEADER_SIZE + SFAT_HEADER_SIZE) as u64
  }

  /// Where the file names start, from the start of the archive.
  pub fn names_offset(&self) -> u64 {
    names_offset(self.sfat.file_count)
  }

  /// Where the file data starts, from the start of the archive.
  pub fn data_offset(&self) -> u64 {
    self.header.data_offset as u64
  }
}

/// Where the file names start in an archive of `file_count` files.
fn names_offset(file_count: u16) -> u64 {
  (SARC_HEADER_SIZE + SFAT_HEADER_SIZE) as u64 + file_count as u64 * SFAT_ENTRY_SIZE + SFNT_HEADER_SIZE as u64
}
