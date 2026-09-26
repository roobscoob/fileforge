pub mod readable;

/// The SFAT header's size.
pub const SFAT_HEADER_SIZE: u16 = 0xC;

/// The most files sead accepts in one archive.
pub const SFAT_MAX_FILES: u16 = 0x3FFF;

pub struct SfatHeader {
  pub file_count: u16,
  pub hash_multiplier: u32,
}
