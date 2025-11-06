use core::num::NonZero;

pub struct FilenameAttributes {
  pub sequence: NonZero<u8>,
  pub hash_index: u32,
}

pub struct SfatEntry {
  pub filename_hash: u32,
  pub filename_attributes: Option<FilenameAttributes>,
  pub start_offset: u32,
  pub end_offset: u32,
}
