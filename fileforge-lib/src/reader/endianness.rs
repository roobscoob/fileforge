#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Endianness {
  Little,
  Big,
}

impl Endianness {
  pub fn inverse(&self) -> Endianness {
    match self {
      Self::Little => Self::Big,
      Self::Big => Self::Little,
    }
  }
}
