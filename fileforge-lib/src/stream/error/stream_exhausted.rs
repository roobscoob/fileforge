use crate::provider::error::out_of_bounds::OutOfBoundsError;

pub struct StreamExhaustedError {
  stream_length: u64,
  read_size: u64,
  read_offset: u64,
}

impl From<OutOfBoundsError> for Option<StreamExhaustedError> {
  fn from(value: OutOfBoundsError) -> Self {
    value.read_length.map(|read_length| StreamExhaustedError {
      stream_length: value.provider_size,
      read_size: read_length,
      read_offset: value.read_offset,
    })
  }
}
