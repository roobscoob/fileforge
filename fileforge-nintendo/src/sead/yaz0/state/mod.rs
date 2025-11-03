pub mod compress;
pub mod malformed_stream;
pub mod reference;

use crate::sead::yaz0::{
  parser::data::{Block, Operation},
  state::{malformed_stream::MalformedStream, reference::ReadbackReference},
};

pub const SEEKBACK_BUFFER_LENGTH: usize = 4096;

#[derive(Clone)]
pub struct Yaz0State {
  unread_bytes: u64,
  offset: u64,
  seekback_buffer: heapless::Deque<u8, SEEKBACK_BUFFER_LENGTH>,
}

impl Yaz0State {
  #[inline]
  pub fn empty() -> Self {
    Yaz0State {
      offset: 0,
      unread_bytes: 0,
      seekback_buffer: heapless::Deque::new(),
    }
  }

  #[inline]
  fn push_byte(&mut self, b: u8) -> () {
    if self.seekback_buffer.is_full() {
      self.seekback_buffer.pop_front();
    }

    let _ = self.seekback_buffer.push_back(b);
  }

  #[inline]
  pub fn readback(&self) -> ReadbackReference<2> {
    self.seekback_buffer.as_slices().into()
  }

  #[inline]
  pub fn last_n(&self, n: usize) -> Option<ReadbackReference<'_, 2>> {
    self.readback().slice((self.readback().len().checked_sub(n)?)..)
  }

  /// Take up to `desired` unread bytes (oldest-first) and decrement `unread_bytes`.
  #[inline]
  pub fn take(&mut self, desired: usize) -> ReadbackReference<'_, 2> {
    let ub = self.unread_bytes as usize;
    let taken = desired.min(ub);
    self.unread_bytes -= taken as u64;
    self.offset += taken as u64;
    self.last_n(ub).unwrap().slice(0..taken).unwrap()
  }

  pub fn drop_all(&mut self) {
    self.offset += self.unread_bytes;
    self.unread_bytes = 0;
  }

  #[inline]
  pub fn offset(&self) -> u64 {
    self.offset
  }

  pub(crate) fn feed_operation(&mut self, operation: Operation) -> Result<(), MalformedStream> {
    match operation {
      Operation::Literal(b) => {
        self.push_byte(b);
        self.unread_bytes += 1;
      }

      Operation::ShortReadback { offset, length } | Operation::LongReadback { offset, length } => {
        for _ in 0..length.get() {
          self.push_byte(
            self
              .last_n(offset.get() as usize)
              .ok_or(MalformedStream::SeekbackOutOfBounds {
                seekback_offset: offset.get(),
                seekback_size: self.seekback_buffer.len() as u16,
              })?
              .get(0)
              .unwrap(),
          )
        }

        self.unread_bytes += length.get() as u64;
      }
    }

    Ok(())
  }

  #[inline]
  pub fn feed(&mut self, block: Block) -> Result<(), MalformedStream> {
    for operation in block.operations.iter() {
      self.feed_operation(*operation)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use core::num::NonZeroU16;

  // ---- Helpers -------------------------------------------------------------

  // Build a Block with exactly one Operation.
  fn blk1(op: Operation) -> Block {
    let mut ops: heapless::Vec<Operation, 8> = heapless::Vec::new();
    ops.push(op).expect("ops capacity");
    Block { operations: ops }
  }

  // Build a Block from a small slice of Operations.
  // (Useful if you want to batch a few ops into a single feed.)
  fn blk(ops_in: &[Operation]) -> Block
  where
    Operation: Clone,
  {
    let mut ops: heapless::Vec<Operation, 8> = heapless::Vec::new();
    for op in ops_in.iter().cloned() {
      ops.push(op).expect("ops capacity");
    }
    Block { operations: ops }
  }

  // Convert a ReadbackReference into a Vec<u8> for easy equality checks.
  fn rb_to_vec(r: ReadbackReference<'_, 2>) -> Vec<u8> {
    let mut out = Vec::with_capacity(r.len());
    for i in 0..r.len() {
      out.push(r.get(i).expect("index must be in-bounds"));
    }
    out
  }

  fn nz(n: u16) -> NonZeroU16 {
    NonZeroU16::new(n).expect("n must be non-zero")
  }

  #[test]
  fn empty_is_quiet() {
    let mut st = Yaz0State::empty();

    // Taking from empty yields empty.
    let taken = st.take(10);
    assert_eq!(taken.len(), 0);
    assert!(rb_to_vec(taken).is_empty());

    // A readback on empty should yield SeekbackOutOfBounds.
    let err = st.feed(blk1(Operation::ShortReadback { offset: nz(1), length: nz(1) })).unwrap_err();
    match err {
      MalformedStream::SeekbackOutOfBounds { seekback_offset, seekback_size } => {
        assert_eq!(seekback_offset, 1);
        assert_eq!(seekback_size, 0);
      }
      other => panic!("unexpected error: {:?}", other),
    }
  }

  #[test]
  fn literals_accumulate_then_take_in_oldest_first_order() {
    let mut st = Yaz0State::empty();

    for &b in b"abcd" {
      st.feed(blk1(Operation::Literal(b))).unwrap();
    }

    // Take 2 -> "ab"
    let t1 = st.take(2);
    assert_eq!(rb_to_vec(t1), b"ab");

    // Remaining unread should be "cd"
    let t2 = st.take(10);
    assert_eq!(rb_to_vec(t2), b"cd");

    // Unread now empty
    let t3 = st.take(1);
    assert_eq!(t3.len(), 0);
  }

  #[test]
  fn take_truncates_to_available() {
    let mut st = Yaz0State::empty();

    st.feed(blk1(Operation::Literal(b'X'))).unwrap(); // unread = 1
    let t = st.take(5);
    assert_eq!(rb_to_vec(t), b"X");

    // Now empty again.
    let t2 = st.take(1);
    assert_eq!(t2.len(), 0);
  }

  #[test]
  fn readback_simple_copy_sliding_source() {
    // Seed with ABCDEF, then readback offset=3 for len=3 should copy DEF.
    let mut st = Yaz0State::empty();

    for &b in b"ABCDEF" {
      st.feed(blk1(Operation::Literal(b))).unwrap();
    }
    st.feed(blk1(Operation::ShortReadback { offset: nz(3), length: nz(3) })).unwrap();

    let out = rb_to_vec(st.take(9999));
    assert_eq!(out, b"ABCDEFDEF");
  }

  #[test]
  fn readback_chained_copies_from_growing_window() {
    let mut st = Yaz0State::empty();

    for &b in b"HELLO" {
      st.feed(blk1(Operation::Literal(b))).unwrap();
    }
    st.feed(blk1(Operation::ShortReadback { offset: nz(2), length: nz(2) })).unwrap();
    st.feed(blk1(Operation::ShortReadback { offset: nz(2), length: nz(2) })).unwrap();

    let out = rb_to_vec(st.take(9999));
    assert_eq!(out, b"HELLOLOLO");
  }

  #[test]
  fn readback_fails_when_offset_exceeds_history() {
    let mut st = Yaz0State::empty();
    for &b in b"AB" {
      st.feed(blk1(Operation::Literal(b))).unwrap();
    }
    // Only 2 bytes history; offset 3 should fail with SeekbackOutOfBounds.
    let err = st.feed(blk1(Operation::ShortReadback { offset: nz(3), length: nz(1) })).unwrap_err();

    match err {
      MalformedStream::SeekbackOutOfBounds { seekback_offset, seekback_size } => {
        assert_eq!(seekback_offset, 3);
        assert_eq!(seekback_size, 2);
      }
      other => panic!("unexpected error: {:?}", other),
    }
  }

  // New: overflow now PANICS. Two tests to cover literal and readback overflows.

  #[test]
  #[should_panic(expected = "Seekback Buffer Overflow")]
  fn overflow_guard_panics_on_literal_beyond_capacity() {
    let mut st = Yaz0State::empty();

    // Seed with some bytes, then fill to capacity exactly via readback.
    st.feed(blk(&[Operation::Literal(b'S'), Operation::Literal(b'E')])).unwrap();

    let remaining = SEEKBACK_BUFFER_LENGTH - 2;
    st.feed(blk1(Operation::ShortReadback {
      offset: nz(1),
      length: nz(remaining as u16),
    }))
    .unwrap(); // equal to capacity is allowed

    // Next literal would exceed capacity -> panic.
    st.feed(blk1(Operation::Literal(b'!'))).unwrap();
  }

  #[test]
  #[should_panic(expected = "Seekback Buffer Overflow")]
  fn overflow_guard_panics_on_readback_beyond_capacity() {
    let mut st = Yaz0State::empty();

    // Seed minimally.
    st.feed(blk1(Operation::Literal(b'X'))).unwrap();

    // Fill to capacity exactly: remaining = capacity - 1
    let remaining = (SEEKBACK_BUFFER_LENGTH - 1) as u16;
    st.feed(blk1(Operation::ShortReadback { offset: nz(1), length: nz(remaining) })).unwrap();

    // One more byte via readback would exceed -> panic.
    st.feed(blk1(Operation::ShortReadback { offset: nz(1), length: nz(1) })).unwrap();
  }

  #[test]
  fn ring_buffer_eviction_without_unread_overflow() {
    let mut st = Yaz0State::empty();

    // Grow history, keep unread tiny by taking immediately.
    let total = SEEKBACK_BUFFER_LENGTH + 7;
    for i in 0..total {
      st.feed(blk1(Operation::Literal((i % 251) as u8))).unwrap();
      let got = rb_to_vec(st.take(1));
      assert_eq!(got, vec![(i % 251) as u8]);
    }

    // Make a distinctive tail, keep unread small.
    for &b in &[9u8, 8, 7, 6, 5] {
      st.feed(blk1(Operation::Literal(b))).unwrap();
      let _ = st.take(1);
    }

    // Copy that tail into unread and read it back.
    st.feed(blk1(Operation::ShortReadback { offset: nz(5), length: nz(5) })).unwrap();
    let rb = rb_to_vec(st.take(10));
    assert_eq!(rb, vec![9, 8, 7, 6, 5]);
  }

  #[test]
  fn mixed_sequence_behaves_as_queue_of_unread() {
    let mut st = Yaz0State::empty();

    for &b in b"abc" {
      st.feed(blk1(Operation::Literal(b))).unwrap();
    }
    let t1 = rb_to_vec(st.take(2));
    assert_eq!(t1, b"ab");

    // unread currently "c"
    st.feed(blk1(Operation::ShortReadback { offset: nz(1), length: nz(2) })).unwrap();

    // unread should be "c", "c", "c"
    let t2 = rb_to_vec(st.take(999));
    assert_eq!(t2, b"ccc");
  }

  #[test]
  fn large_readback_exact_capacity_then_drain() {
    let mut st = Yaz0State::empty();

    for &b in b"XY" {
      st.feed(blk1(Operation::Literal(b))).unwrap();
    }

    // Fill unread to capacity exactly with an offset=1 run.
    let remaining = SEEKBACK_BUFFER_LENGTH - 2;
    st.feed(blk1(Operation::ShortReadback {
      offset: nz(1),
      length: nz(remaining as u16),
    }))
    .unwrap();

    // Taking everything yields length == capacity, starts with "XY", then a run of 'Y's.
    let all = rb_to_vec(st.take(usize::MAX));
    assert_eq!(all.len(), SEEKBACK_BUFFER_LENGTH);
    assert_eq!(&all[..2], b"XY");
    assert!(all[2..].iter().all(|&b| b == b'Y'));
  }

  #[test]
  fn readback_offset_one_is_repeat_last_byte() {
    let mut st = Yaz0State::empty();
    for &b in b"AB" {
      st.feed(blk1(Operation::Literal(b))).unwrap();
    }
    // Repeat last byte 'B' five times.
    st.feed(blk1(Operation::ShortReadback { offset: nz(1), length: nz(5) })).unwrap();

    // Result is "A" + 6×"B"
    let out = rb_to_vec(st.take(usize::MAX));
    assert_eq!(out, b"ABBBBBB");
  }

  #[test]
  fn readback_exact_window_boundary() {
    let mut st = Yaz0State::empty();

    for &b in b"QRST" {
      st.feed(blk1(Operation::Literal(b))).unwrap();
    }
    // len == 4; offset == 4 is permitted. It should copy from 'Q','R','S','T'.
    st.feed(blk1(Operation::ShortReadback { offset: nz(4), length: nz(4) })).unwrap();

    let out = rb_to_vec(st.take(usize::MAX));
    assert_eq!(out, b"QRSTQRST");
  }

  #[test]
  fn taking_zero_is_noop_and_does_not_panic() {
    let mut st = Yaz0State::empty();
    for &b in b"HI" {
      st.feed(blk1(Operation::Literal(b))).unwrap();
    }

    let t0 = st.take(0);
    assert_eq!(t0.len(), 0);

    let remaining = rb_to_vec(st.take(2));
    assert_eq!(remaining, b"HI");
  }
}
