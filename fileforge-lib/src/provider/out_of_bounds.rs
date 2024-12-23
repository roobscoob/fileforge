#[derive(Debug)]
pub struct SliceOutOfBoundsError {
  pub read_offset: u64,
  pub read_size: Option<u64>,
  pub provider_size: u64,
}

impl SliceOutOfBoundsError {
  pub fn assert_in_bounds(
    read_offset: u64,
    read_size: Option<u64>,
    provider_size: u64,
  ) -> Result<u64, SliceOutOfBoundsError> {
    let error = SliceOutOfBoundsError {
      read_offset,
      read_size,
      provider_size,
    };

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
    self
      .read_size
      .map(|v| self.read_offset.checked_add(v))
      .unwrap_or(Some(self.provider_size))
  }
}
