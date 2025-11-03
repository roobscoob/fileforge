use std::num::NonZero;

use crate::sead::yaz0::state::{malformed_stream::MalformedStream, Yaz0State};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Block {
  pub(crate) operations: heapless::Vec<Operation, 8>,
}

impl Block {
  pub fn of(operation: Operation) -> Block {
    Self {
      operations: heapless::Vec::from_slice(&[operation]).unwrap(),
    }
  }

  pub fn empty() -> Block {
    Self { operations: heapless::Vec::new() }
  }

  pub fn len(&self) -> u16 {
    self.operations.iter().map(|v| v.len()).sum()
  }

  pub fn is_empty(&self) -> bool {
    self.operations.is_empty()
  }

  pub fn is_full(&self) -> bool {
    self.operations.is_full()
  }

  pub fn compute_header(&self) -> u8 {
    let mut header = 0;

    for (i, new) in self.operations.iter().enumerate() {
      if let Operation::Literal(..) = new {
        header |= 1 << (7 - i)
      }
    }

    header
  }

  pub fn split_at_with_post(self, offset: u64, post_block_state: &Yaz0State) -> Result<(Block, Option<u8>, Option<u8>, Block), MalformedStream> {
    let total = self.len() as u64;

    if offset >= total {
      return Ok((self, None, None, Block::empty()));
    }

    if offset == 0 {
      return Ok((Block::empty(), None, None, self));
    }

    let mut left = Block { operations: heapless::Vec::new() };
    let mut right = Block { operations: heapless::Vec::new() };

    let mut acc: u64 = 0;
    let mut opt = None;

    let mut read = post_block_state.last_n(self.len() as usize).unwrap().into_iter();

    for &op in &self.operations {
      let op_len = op.len() as u64;

      // Entirely to the right
      if offset <= acc {
        if right.operations.is_full() {
          let Operation::Literal(byte) = right.operations.remove(0) else {
            panic!("What the fuck");
          };

          right.operations.push(op).unwrap();

          return Ok((left, opt, Some(byte), right));
        };

        right.operations.push(op).unwrap();
        acc += op_len;
        continue;
      }

      // Entirely to the left
      if offset >= acc + op_len {
        read.nth(op_len as usize - 1);
        acc += op_len;
        left.operations.push(op).unwrap();
        continue;
      }

      // Split occurs within this operation

      let k = (offset - acc) as u16; // 0 < k < op_len for readbacks; 0 or 1 for literal
      acc += op_len;

      match op {
        Operation::Literal(b) => {
          read.next();

          // Literals have len == 1; k is 0 or 1.
          if k == 0 {
            right.operations.push(Operation::Literal(b)).unwrap();
          } else {
            left.operations.push(Operation::Literal(b)).unwrap();
          }
        }

        Operation::ShortReadback { offset: back, length } | Operation::LongReadback { offset: back, length } => {
          // First k bytes go left, remainder goes right.
          if k > 0 {
            if k == 1 {
              left.operations.push(Operation::Literal(read.next().unwrap())).unwrap();
            } else if k == 2 {
              left.operations.push(Operation::Literal(read.next().unwrap())).unwrap();

              if left.operations.is_full() {
                opt = Some(read.next().unwrap());
              } else {
                left.operations.push(Operation::Literal(read.next().unwrap())).unwrap();
              }
            } else {
              read.nth(k as usize - 1);
              left.operations.push(Operation::readback(back.get(), k).unwrap()).unwrap();
            }
          }
          let rem = length.get() - k;
          if rem > 0 {
            if rem == 1 {
              right.operations.push(Operation::Literal(read.next().unwrap())).unwrap();
            } else if rem == 2 {
              right.operations.push(Operation::Literal(read.next().unwrap())).unwrap();
              right.operations.push(Operation::Literal(read.next().unwrap())).unwrap();
            } else {
              right.operations.push(Operation::readback(back.get(), rem).unwrap()).unwrap();
            }
          }
        }
      }
    }

    Ok((left, opt, None, right))
  }

  /// Build the left block ([0, offset)) and feed `pre_block_state` exactly
  /// through those bytes (no further).
  pub fn split_at_with_pre(self, offset: u64, pre_block_state: &mut Yaz0State) -> Result<(Block, Option<u8>, Option<u8>, Block), MalformedStream> {
    let total = self.len() as u64;

    if offset == 0 {
      return Ok((Block::empty(), None, None, self));
    }

    if offset >= total {
      pre_block_state.feed(self.clone())?;
      return Ok((self, None, None, Block::empty()));
    }

    let mut opt = None;

    let mut left = Block { operations: heapless::Vec::new() };
    let mut right = Block { operations: heapless::Vec::new() };

    // Running decoded position within this block.
    let mut acc: u64 = 0;

    for &op in &self.operations {
      let op_len = op.len() as u64;

      // Entirely to the right → stop; nothing more contributes to left.
      if offset <= acc {
        if right.operations.is_full() {
          let Operation::Literal(byte) = right.operations.remove(0) else {
            panic!("What the fuck");
          };

          right.operations.push(op).unwrap();

          return Ok((left, opt, Some(byte), right));
        };

        right.operations.push(op).unwrap();
        acc += op_len;
        continue;
      }

      // Entirely to the left → emit unchanged, feed unchanged.
      if offset >= acc + op_len {
        left.operations.push(op).unwrap();
        pre_block_state.feed_operation(op).expect("malformed stream while feeding left segment");
        acc += op_len;
        continue;
      }

      // Split occurs within this operation.
      let k = (offset - acc) as u16; // 0 < k < op_len for readbacks; 0 or 1 for literal
      acc += op_len;

      match op {
        Operation::Literal(b) => {
          if k == 0 {
            right.operations.push(Operation::Literal(b)).unwrap();
          } else {
            let lop = Operation::Literal(b);
            left.operations.push(lop).unwrap();
            pre_block_state.feed_operation(lop).expect("literals are never malformed");
          }
        }

        Operation::ShortReadback { offset: back, length } | Operation::LongReadback { offset: back, length } => {
          if k > 0 {
            if k == 1 {
              let mut values = pre_block_state.last_n(back.get() as usize).unwrap().cycle().take(length.get() as usize);
              let lop = Operation::Literal(values.next().unwrap());
              pre_block_state.feed_operation(lop).unwrap();
              left.operations.push(lop).unwrap();
            } else if k == 2 {
              let mut values = pre_block_state.last_n(back.get() as usize).unwrap().cycle().take(length.get() as usize);
              let lop = Operation::Literal(values.next().unwrap());
              pre_block_state.feed_operation(lop).unwrap();
              left.operations.push(lop).unwrap();

              let mut values = pre_block_state.last_n(back.get() as usize).unwrap().cycle().take(length.get() as usize);
              let b = values.next().unwrap();
              let lop = Operation::Literal(b);
              pre_block_state.feed_operation(lop).unwrap();

              if left.operations.is_full() {
                opt = Some(b);
              } else {
                left.operations.push(lop).unwrap();
              }
            } else {
              // k >= 3 → keep a readback op for the left portion.
              let lop = Operation::readback(back.get(), k).unwrap();
              left.operations.push(lop).unwrap();
              pre_block_state.feed_operation(lop).expect("malformed stream while feeding split-readback left portion");
            }
          }

          let mut values = pre_block_state.last_n(back.get() as usize).unwrap().cycle().take(length.get() as usize);

          let rem = length.get() - k;
          if rem > 0 {
            if rem == 1 || rem == 2 {
              // Literalize the first k bytes from the pre-window.
              for _ in 0..rem {
                let lop = Operation::Literal(values.next().unwrap());
                right.operations.push(lop).unwrap();
              }
            } else {
              // k >= 3 → keep a readback op for the left portion.
              right.operations.push(Operation::readback(back.get(), rem).unwrap()).unwrap();
            }
          }
        }
      }
    }

    Ok((left, opt, None, right))
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
  Literal(u8),
  ShortReadback { offset: NonZero<u16>, length: NonZero<u16> },
  LongReadback { offset: NonZero<u16>, length: NonZero<u16> },
}

impl Operation {
  pub fn lit(v: u8) -> Operation {
    Operation::Literal(v)
  }

  pub fn readback(offset: u16, length: u16) -> Option<Operation> {
    if length < 3 {
      return None;
    }

    let offset = NonZero::new(offset)?;
    let length = NonZero::new(length)?;

    Some(if length.get() < 0x12 {
      Self::ShortReadback { offset, length }
    } else {
      Self::LongReadback { offset, length }
    })
  }

  pub fn len(self) -> u16 {
    match self {
      Self::Literal(..) => 1,
      Self::ShortReadback { length, .. } => length.get(),
      Self::LongReadback { length, .. } => length.get(),
    }
  }

  pub fn encoded_len(self) -> u8 {
    match self {
      Self::Literal(..) => 1,
      Self::ShortReadback { .. } => 2,
      Self::LongReadback { .. } => 3,
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
