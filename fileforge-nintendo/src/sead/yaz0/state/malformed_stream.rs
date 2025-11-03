#[derive(Debug)]
pub enum MalformedStream {
  SeekbackOutOfBounds { seekback_offset: u16, seekback_size: u16 },
}
