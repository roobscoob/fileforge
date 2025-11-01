#[derive(Debug)]
pub enum MalformedStream {
  EmptySeekback,
  SeekbackOutOfBounds { seekback_offset: u16, seekback_size: u16 },
}
