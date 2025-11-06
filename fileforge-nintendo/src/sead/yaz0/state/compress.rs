use core::{cmp::min, slice::SliceIndex};

use crate::sead::yaz0::{
  parser::data::Block,
  state::{reference::ReadbackReference, Operation, Yaz0State},
};

impl Yaz0State {
  pub fn compress_block<const C: usize>(&self, data: &mut ReadbackReference<C>) -> Block {
    let mut operations = heapless::Vec::<Operation, 8>::new();

    while let Some(operation) = self.compress(data) {
      operations.push(operation).map_err(|_| {}).unwrap();
    }

    Block { operations }
  }

  pub(crate) fn compress<const C: usize>(&self, data: &mut ReadbackReference<C>) -> Option<Operation> {
    // If no history or not enough lookahead for a match, emit a literal.
    if self.seekback_buffer.len() == 0 || data.len() < 3 {
      let b = data.get(0)?;
      *data = data.slice(1..).unwrap();
      return Some(Operation::lit(b));
    }

    let max_len = min(data.len(), 0x111);

    let best = (1..=self.seekback_buffer.len())
      .filter_map(|off| {
        let base = self.last_n(off)?;

        Operation::readback(off as u16, (0..max_len).map(|i| (data[i], base.get(i % off).unwrap())).take_while(|(a, b)| *a == *b).count() as u16)
      })
      .max_by_key(|v| v.len())
      .unwrap_or_else(|| Operation::Literal(data.get(0).unwrap()));

    *data = data.slice(best.len() as usize..).unwrap();
    Some(best)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Helper: prime the seekback buffer with bytes (newest last).
  fn seed_history(state: &mut Yaz0State, bytes: &[u8]) {
    for &b in bytes {
      state.push_byte(b);
    }
  }

  #[test]
  fn none_when_input_empty() {
    let st = Yaz0State::empty();
    let mut data: &[u8] = &[];
    assert!(st.compress(&mut ReadbackReference::of(data)).is_none(), "empty input should yield None");
  }

  #[test]
  fn literal_when_no_history() {
    let st = Yaz0State::empty();
    let mut data = ReadbackReference::of(b"AB");
    let op = st.compress(&mut data).expect("some op");
    match op {
      Operation::Literal(b) => assert_eq!(b, b'A'),
      _ => panic!("expected Literal"),
    }
    assert_eq!(data.len(), 1, "must consume exactly one byte for a literal");
    assert_eq!(data.get(0).unwrap(), 0x42, "must consume exactly one byte for a literal");
  }

  #[test]
  fn literal_when_lookahead_lt_3_even_with_history() {
    let mut st = Yaz0State::empty();
    seed_history(&mut st, b"XYZ"); // history exists
    let mut data = ReadbackReference::of(b"Q!"); // only 2 bytes of lookahead
    let op = st.compress(&mut data).expect("some op");
    match op {
      Operation::Literal(b) => assert_eq!(b, b'Q'),
      _ => panic!("expected Literal"),
    }
    assert_eq!(data.len(), 1, "consumed one byte");
    assert_eq!(data.get(0).unwrap(), 0x21, "consumed one byte");
  }

  #[test]
  fn overlapping_match_one_byte_offset() {
    // History ends with 'A' so offset=1 should repeat 'A's.
    let mut st = Yaz0State::empty();
    seed_history(&mut st, b"A");
    let mut data = ReadbackReference::of(b"AAAAA"); // 5 A's available

    let op = st.compress(&mut data).expect("some op");
    match op {
      Operation::ShortReadback { offset, length } => {
        assert_eq!(offset.get(), 1, "periodic 1-byte window");
        assert_eq!(length.get(), 5, "should extend across overlap");
      }
      _ => panic!("expected ShortReadback"),
    }
    assert_eq!(data.len(), 0, "consumed all 5 bytes");
  }

  #[test]
  fn chooses_longest_match() {
    // Seed a tail that contains "ABCDABCD" so distance 4 gives a long match.
    let mut st = Yaz0State::empty();
    seed_history(&mut st, b"ZZZZABCDABCD");
    let mut data = ReadbackReference::of(b"ABCDABCDX"); // best match length = 8

    let op = st.compress(&mut data).expect("some op");
    match op {
      Operation::ShortReadback { offset, length } => {
        assert!(offset.get() == 4 || offset.get() == 8, "offset should be a divisor of the repeated pattern (got {})", offset);
        assert_eq!(length.get(), 8, "must pick the longest match");
      }
      v => panic!("expected ShortReadback, got: {v:?}"),
    }
    assert_eq!(data.len(), 1, "consumed one byte");
    assert_eq!(data.get(0).unwrap(), 0x58, "consumed one byte");
  }

  #[test]
  fn cap_length_at_0x111() {
    // History allows repeating 'A' forever (offset=1). Input has 300 'A's.
    let mut st = Yaz0State::empty();
    seed_history(&mut st, b"A");
    let mut data_vec = vec![b'A'; 300];
    let mut data = ReadbackReference::of(&data_vec);

    let op = st.compress(&mut data).expect("some op");
    match op {
      Operation::LongReadback { offset, length } | Operation::ShortReadback { offset, length } => {
        assert_eq!(offset.get(), 1, "still periodic single-byte source");
        assert_eq!(length.get(), 0x111, "Yaz0 backref max should be 0x111 (273)");
      }
      _ => panic!("expected a readback"),
    }
    assert_eq!(data.len(), 300 - 0x111, "remaining should be input - capped length");
  }

  #[test]
  fn no_match_falls_back_to_literal() {
    // History contains bytes that don't start any 3-gram matching "Q.."
    let mut st = Yaz0State::empty();
    seed_history(&mut st, b"ABCDEFGH");
    let mut data = ReadbackReference::of(b"QRSXYZ");

    let op = st.compress(&mut data).expect("some op");
    match op {
      Operation::Literal(b) => assert_eq!(b, b'Q'),
      _ => panic!("expected Literal"),
    }
    assert_eq!(data.collect::<heapless::Vec<u8, 5>>(), b"RSXYZ");
  }

  #[test]
  fn readback_vs_literal_boundary_at_3() {
    // Ensure matches <3 do NOT produce readbacks.
    let mut st = Yaz0State::empty();
    seed_history(&mut st, b"AB"); // only 2 bytes periodic pattern
    let mut data = ReadbackReference::of(b"ABZ"); // only first two match, third differs

    let op = st.compress(&mut data).expect("some op");
    match op {
      Operation::Literal(b) => assert_eq!(b, b'A'),
      _ => panic!("expected Literal because match < 3"),
    }
    assert_eq!(data.collect::<heapless::Vec<u8, 5>>(), b"BZ");
  }
}
