#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endianness {
  LittleEndian,
  BigEndian,
}

impl Endianness {
  pub fn swap(self) -> Endianness {
    match self {
      Self::BigEndian => Self::LittleEndian,
      Self::LittleEndian => Self::BigEndian,
    }
  }
}
