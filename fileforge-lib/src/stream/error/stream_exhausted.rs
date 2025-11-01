use crate::provider::error::out_of_bounds::OutOfBoundsError;

#[derive(Debug)]
pub struct StreamExhaustedError {
  pub stream_length: u64,
  pub read_length: u64,
  pub read_offset: u64,
}

impl From<OutOfBoundsError> for Option<StreamExhaustedError> {
  fn from(value: OutOfBoundsError) -> Self {
    value.read_length.map(|read_length| StreamExhaustedError {
      stream_length: value.provider_size,
      read_length,
      read_offset: value.read_offset,
    })
  }
}

impl StreamExhaustedError {
  pub fn assert(stream_length: u64, read_offset: u64, read_length: u64) -> Result<(), Self> {
    let read_end = read_offset.checked_add(read_length).ok_or(Self {
      read_offset,
      read_length,
      stream_length,
    })?;

    if read_end > stream_length {
      Err(Self {
        read_offset,
        read_length,
        stream_length,
      })
    } else {
      Ok(())
    }
  }
}
