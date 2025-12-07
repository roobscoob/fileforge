pub mod readable;

use fileforge::{binary_reader::endianness::Endianness, stream::ReadableStream};

use crate::sead::sarc::sfat::SfatTable;

pub struct SarcHeader<'pool, UnderlyingStream: ReadableStream<Type = u8>> {
  pub endianness: Endianness,
  pub size: u32,
  pub data_section_offset: u32,
  pub version: (u8, u8),
  pub sfat_table: SfatTable<'pool, UnderlyingStream>,
}
