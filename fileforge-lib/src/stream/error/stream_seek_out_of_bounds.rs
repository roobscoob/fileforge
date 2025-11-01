#[derive(Debug)]
pub struct StreamSeekOutOfBoundsError {
  pub stream_length: u64,
  pub seek_point: u64,
}

impl StreamSeekOutOfBoundsError {
  pub fn assert(stream_length: u64, seek_point: u64) -> Result<(), Self> {
    if seek_point > stream_length { Err(Self { seek_point, stream_length }) } else { Ok(()) }
  }
}
