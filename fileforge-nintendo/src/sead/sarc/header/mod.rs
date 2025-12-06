pub mod readable;

use fileforge::binary_reader::endianness::Endianness;

pub struct SarcHeader {
  pub endianness: Endianness,
  pub size: u32,
  pub data_section_offset: u32,
  pub version: (u8, u8),
}
