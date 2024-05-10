pub struct SliceOutOfBoundsError {
  pub(crate) read_offset: u64,
  pub(crate) read_size: u64,
  pub(crate) provider_size: u64,
}

impl SliceOutOfBoundsError {
  pub fn assert_in_bounds(read_offset: u64, read_size: u64, provider_size: u64) -> Result<u64, SliceOutOfBoundsError> {
    let error = SliceOutOfBoundsError { read_offset, read_size, provider_size };

    if let Some(read_end) = error.read_end() {
      if read_end > provider_size {
        //  Out of bounds due to... out of bounds... lol
        Err(error)
      } else {
        Ok(read_end)
      }
    } else {
      //  Out of bounds due to overflow in (offset + size)
      Err(error)
    }
  }

  pub fn read_end(&self) -> Option<u64> {
    self.read_offset.checked_add(self.read_size)
  }
}