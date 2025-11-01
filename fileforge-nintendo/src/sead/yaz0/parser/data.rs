use std::num::NonZero;

#[derive(Clone, Copy, Debug)]
pub enum Operation {
  Literal(u8),
  Readback { offset: u16, length: NonZero<u16> },
}

impl Operation {
  pub fn len(self) -> u16 {
    match self {
      Self::Literal(..) => 1,
      Self::Readback { length, .. } => length.get(),
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BlockHeader {
  bits: u8,
  mask: u8,
}

impl BlockHeader {
  #[inline]
  pub fn empty() -> BlockHeader {
    BlockHeader { bits: 0, mask: 0 }
  }

  #[inline]
  pub fn from_byte(byte: u8) -> BlockHeader {
    BlockHeader { bits: byte, mask: 0x80 }
  }

  #[inline]
  pub fn peek(&self) -> Option<bool> {
    if self.mask == 0 {
      None
    } else {
      Some((self.bits & self.mask) != 0)
    }
  }

  #[inline]
  pub fn take(&mut self) -> Option<bool> {
    let bit = self.peek()?;
    self.mask >>= 1; // advance MSB→LSB
    Some(bit)
  }

  #[inline]
  pub fn is_exhausted(&self) -> bool {
    self.mask == 0
  }
}
