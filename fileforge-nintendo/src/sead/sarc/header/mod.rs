pub mod readable;

use fileforge::binary_reader::endianness::Endianness;

/// The SARC header's size, which is also where the SFAT starts.
pub const SARC_HEADER_SIZE: u16 = 0x14;

/// The only SARC version sead accepts.
pub const SARC_VERSION: u16 = 0x100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SarcHeader {
  pub endianness: Endianness,
  pub file_size: u32,
  /// Where the file data starts, from the start of the archive.
  pub data_offset: u32,
  pub version: u16,
}
