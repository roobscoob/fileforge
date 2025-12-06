use core::num::NonZero;

use fileforge_macros::{text, FileforgeError};

pub struct FilenameAttributes {
  pub sequence: NonZero<u8>,
  pub hash_index: u32,
}

#[derive(FileforgeError)]
pub enum FilenameAttributesError {
  #[report(&text!("Encountered a 'FilenameAttribute' with a zero sequence."))]
  ZeroSequence,
}

impl FilenameAttributes {
  pub fn from_bits(value: u32) -> Result<Option<FilenameAttributes>, FilenameAttributesError> {
    if value == 0 {
      Ok(None)
    } else {
      let sequence = NonZero::new((value >> 24) as u8).ok_or(FilenameAttributesError::ZeroSequence)?;
      let hash_index = value & 0xFFFFFF;

      Ok(Some(FilenameAttributes { sequence, hash_index }))
    }
  }
}
