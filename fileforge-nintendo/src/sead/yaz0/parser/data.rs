use core::num::NonZero;

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
    self.operations.len() == 8
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

  /// Splits the block at `offset`, a decoded position within it, without feeding any state.
  ///
  /// `state_at_offset` is the history as of `offset`: it must already include every byte this
  /// block decodes to before `offset`. That's what `split_at_with_pre(offset, …)` leaves behind.
  pub fn split_at_with_mid(self, offset: u64, state_at_offset: &Yaz0State) -> Result<(Block, Option<u8>, Option<u8>, Block), MalformedStream> {
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
        acc += op_len;
        left.operations.push(op).unwrap();
        continue;
      }

      // Split occurs within this operation

      let k = (offset - acc) as u16; // 0 < k < op_len for readbacks; 0 or 1 for literal
      acc += op_len;

      match op {
        Operation::Literal(b) => {
          // Literals have len == 1; k is 0 or 1.
          if k == 0 {
            right.operations.push(Operation::Literal(b)).unwrap();
          } else {
            left.operations.push(Operation::Literal(b)).unwrap();
          }
        }

        Operation::ShortReadback { offset: back, length } | Operation::LongReadback { offset: back, length } => {
          // The history ends at the split point, so it already holds the `k` bytes this operation
          // decoded to before the split, and the operation continues from `back` bytes behind the
          // split point.

          // The first k bytes go left. 1 or 2 bytes can't be a readback (the minimum length is 3),
          // so they become literals: the last k bytes of the history.
          if k == 1 || k == 2 {
            let mut before = state_at_offset.last_n(k as usize).unwrap();
            left.operations.push(Operation::Literal(before.next().unwrap())).unwrap();

            if k == 2 {
              let second = before.next().unwrap();

              if left.operations.is_full() {
                opt = Some(second);
              } else {
                left.operations.push(Operation::Literal(second)).unwrap();
              }
            }
          } else if k >= 3 {
            left.operations.push(Operation::readback(back.get(), k).unwrap()).unwrap();
          }

          // The remainder goes right. If it's 1 or 2 bytes it becomes literals, starting `back`
          // bytes behind the split point. It repeats every `back` bytes when it overlaps itself
          // (`back` smaller than the remainder), hence `cycle`.
          let rem = length.get() - k;
          if rem == 1 || rem == 2 {
            let mut after = state_at_offset.last_n(back.get() as usize).unwrap().cycle();

            for _ in 0..rem {
              right.operations.push(Operation::Literal(after.next().unwrap())).unwrap();
            }
          } else if rem >= 3 {
            right.operations.push(Operation::readback(back.get(), rem).unwrap()).unwrap();
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

#[cfg(test)]
mod tests {
  use std::{vec, vec::Vec};

  use super::{Block, Operation};
  use crate::sead::yaz0::state::Yaz0State;

  fn lit(byte: u8) -> Operation {
    Operation::Literal(byte)
  }

  fn rb(back: u16, length: u16) -> Operation {
    Operation::readback(back, length).unwrap()
  }

  fn block(ops: &[Operation]) -> Block {
    Block {
      operations: heapless::Vec::from_slice(ops).unwrap(),
    }
  }

  /// A state whose history is `bytes`, all already taken.
  fn history(bytes: &[u8]) -> Yaz0State {
    let mut state = Yaz0State::empty();

    for &byte in bytes {
      state.feed_operation(lit(byte)).unwrap();
    }

    state.take(usize::MAX);
    state
  }

  fn decode(state: &mut Yaz0State, ops: &[Operation]) -> Vec<u8> {
    let mut out = Vec::new();

    for &op in ops {
      state.feed_operation(op).unwrap();
      out.extend(state.take(usize::MAX));
    }

    out
  }

  #[test]
  fn a_two_byte_remainder_becomes_the_bytes_after_the_split() {
    // "ABCD", then "copy 4 from 4 back": ABCDABCD. Split at 6, inside the copy (2 bytes in, 2 left).
    let ops = [lit(b'A'), lit(b'B'), lit(b'C'), lit(b'D'), rb(4, 4)];
    let (left, opt, underflow, right) = block(&ops).split_at_with_mid(6, &history(b"ABCDAB")).unwrap();

    assert_eq!(&left.operations[..], &[lit(b'A'), lit(b'B'), lit(b'C'), lit(b'D'), lit(b'A'), lit(b'B')]);
    assert_eq!((opt, underflow), (None, None));
    assert_eq!(&right.operations[..], &[lit(b'C'), lit(b'D')]);
  }

  #[test]
  fn a_one_byte_remainder_becomes_the_byte_after_the_split() {
    // Split at 7: 3 bytes into the copy stay a readback, 1 is left.
    let ops = [lit(b'A'), lit(b'B'), lit(b'C'), lit(b'D'), rb(4, 4)];
    let (left, _, _, right) = block(&ops).split_at_with_mid(7, &history(b"ABCDABC")).unwrap();

    assert_eq!(&left.operations[..], &[lit(b'A'), lit(b'B'), lit(b'C'), lit(b'D'), rb(4, 3)]);
    assert_eq!(&right.operations[..], &[lit(b'D')]);
  }

  #[test]
  fn a_remainder_that_overlaps_itself_repeats() {
    // "AB", then "copy 4 from 1 back": ABBBBB. At 4 the history's last byte repeats for both remaining bytes.
    let ops = [lit(b'A'), lit(b'B'), rb(1, 4)];
    let (_, _, _, right) = block(&ops).split_at_with_mid(4, &history(b"ABBB")).unwrap();

    assert_eq!(&right.operations[..], &[lit(b'B'), lit(b'B')]);
  }

  #[test]
  fn every_split_point_decodes_to_the_original_bytes() {
    // (history before the block, the block's operations)
    let seven: Vec<Operation> = b"ABCDEFG".iter().map(|&b| lit(b)).collect();
    let cases: Vec<(&[u8], Vec<Operation>)> = vec![
      (b"", vec![lit(b'A'), lit(b'B'), lit(b'C'), lit(b'D'), rb(4, 4)]),
      // A split 3 bytes in with 2 left: shifted reads give the wrong bytes here.
      (b"", vec![lit(b'A'), lit(b'B'), rb(2, 5)]),
      // Overlapping copies, which need the history to repeat.
      (b"", vec![lit(b'A'), lit(b'B'), rb(1, 4)]),
      (b"", vec![lit(b'A'), lit(b'B'), lit(b'C'), rb(3, 20)]),
      // Seven literals then a readback: a 2-byte split leaves the left half full (the `opt` byte).
      (b"", [&seven[..], &[rb(3, 4)]].concat()),
      // A readback first, then seven literals: a 2-byte remainder overfills the right half (the
      // underflow byte).
      (b"XYZ", [&[rb(3, 5)], &seven[..]].concat()),
    ];

    // Make sure the rare paths are really exercised.
    let (mut opt_seen, mut underflow_seen) = (false, false);

    for (before, ops) in cases {
      let expected = decode(&mut history(before), &ops);

      for offset in 0..=expected.len() {
        let before_offset = [before, &expected[..offset]].concat();
        let (left, opt, underflow, right) = block(&ops).split_at_with_mid(offset as u64, &history(&before_offset)).unwrap();

        opt_seen |= opt.is_some();
        underflow_seen |= underflow.is_some();

        let mut decoded_left = decode(&mut history(before), &left.operations);
        decoded_left.extend(opt);
        assert_eq!(decoded_left, &expected[..offset], "left half of {ops:?} split at {offset}");

        let mut state = history(&before_offset);
        let mut decoded_right = Vec::new();

        if let Some(byte) = underflow {
          decoded_right.push(byte);
          state = history(&[&before_offset[..], &[byte]].concat());
        }

        decoded_right.extend(decode(&mut state, &right.operations));
        assert_eq!(decoded_right, &expected[offset..], "right half of {ops:?} split at {offset}");
      }
    }

    assert!(opt_seen && underflow_seen, "the cases should cover a full left half and a full right half");
  }
}
