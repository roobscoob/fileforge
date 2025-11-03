use std::ops::{Bound, Index, RangeBounds};

fn normalize_bounds<R: RangeBounds<usize>>(r: R, len: usize) -> Option<(usize, usize)> {
  let start = match r.start_bound() {
    Bound::Included(&i) => Some(i),
    Bound::Excluded(&i) => i.checked_add(1),
    Bound::Unbounded => Some(0),
  }?;
  let end = match r.end_bound() {
    Bound::Included(&i) => i.checked_add(1),
    Bound::Excluded(&i) => Some(i),
    Bound::Unbounded => Some(len),
  }?;
  (start <= end && end <= len).then_some((start, end))
}

#[derive(Clone, Copy)]
pub struct ReadbackReference<'l, const C: usize> {
  parts: [&'l [u8]; C],
}

impl<'l> From<&'l [u8]> for ReadbackReference<'l, 1> {
  fn from(value: &'l [u8]) -> Self {
    Self { parts: [value] }
  }
}

impl<'l> From<(&'l [u8], &'l [u8])> for ReadbackReference<'l, 2> {
  fn from(value: (&'l [u8], &'l [u8])) -> Self {
    Self { parts: [value.0, value.1] }
  }
}

impl<'l> ReadbackReference<'l, 1> {
  #[inline]
  pub fn of(data: &'l [u8]) -> Self {
    Self { parts: [data] }
  }
}

impl<'l, const C: usize> ReadbackReference<'l, C> {
  #[inline]
  pub fn empty() -> Self {
    Self { parts: [const { &[] }; C] }
  }

  /// Raw constructor from exact array.
  #[inline]
  pub fn from_parts(parts: [&'l [u8]; C]) -> Self {
    Self { parts }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.parts.iter().map(|p| p.len()).sum()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.parts.iter().all(|p| p.is_empty())
  }

  #[inline]
  pub fn get(&self, mut i: usize) -> Option<u8> {
    for seg in &self.parts {
      if i < seg.len() {
        return Some(seg[i]);
      }
      i -= seg.len();
    }
    None
  }

  /// Returns a view with the same `C`, where segments outside the slice are `&[]`
  /// and segments partially overlapped are truncated to the overlapped window.
  #[inline]
  pub fn slice<R: RangeBounds<usize>>(&self, r: R) -> Option<Self> {
    let (start, end) = normalize_bounds(r, self.len())?;
    debug_assert!(start <= end, "invalid slice range");

    if start == end {
      return Some(Self::empty());
    }

    // Walk parts to compute overlaps.
    let mut out = [const { (&[]) as &[u8] }; C];
    let mut acc = 0usize;
    for (k, seg) in self.parts.iter().enumerate() {
      let seg_len = seg.len();
      let seg_start = acc;
      let seg_end = acc + seg_len;

      // overlap with [start, end)
      let lo = start.max(seg_start);
      let hi = end.min(seg_end);
      if lo < hi {
        let s = lo - seg_start;
        let e = hi - seg_start;
        out[k] = &seg[s..e];
      } else {
        out[k] = &[];
      }
      acc = seg_end;
      if acc >= end {
        // We can early-exit when we've covered the window.
        // But we still assigned the current segment; the rest stay empty.
        break;
      }
    }
    Some(Self { parts: out })
  }

  /// Expose parts for debugging/tests if needed.
  #[inline]
  pub fn parts(&self) -> [&'l [u8]; C] {
    self.parts
  }
}

impl<'l, const C: usize> From<[&'l [u8]; C]> for ReadbackReference<'l, C> {
  #[inline]
  fn from(parts: [&'l [u8]; C]) -> Self {
    Self::from_parts(parts)
  }
}

impl<'l, const C: usize> Iterator for ReadbackReference<'l, C> {
  type Item = u8;

  fn next(&mut self) -> Option<Self::Item> {
    // Find first non-empty segment, then pop its head.
    for seg in &mut self.parts {
      if !seg.is_empty() {
        let (hd, rest) = seg.split_first().expect("checked non-empty");
        *seg = rest;
        return Some(*hd);
      }
    }
    None
  }
}

impl<'l, const C: usize> Index<usize> for ReadbackReference<'l, C> {
  type Output = u8;

  fn index(&self, mut i: usize) -> &Self::Output {
    for seg in &self.parts {
      if i < seg.len() {
        return &seg[i];
      }
      i -= seg.len();
    }
    panic!("index out of bounds: the len is {} but the index is {}", self.len(), i);
  }
}

/* ------------------------- Tests (adapted) ------------------------- */

#[cfg(test)]
mod tests {
  use super::*;
  use core::ops::{Bound, RangeBounds};

  type Readback2<'a> = ReadbackReference<'a, 2>;

  fn collect<const C: usize>(rb: ReadbackReference<'_, C>) -> Vec<u8> {
    rb.into_iter().collect()
  }

  fn rb2<'a>(l: &'a [u8], r: &'a [u8]) -> Readback2<'a> {
    Readback2::from([l, r])
  }

  #[test]
  fn empty_len_and_is_empty() {
    let r = Readback2::empty();
    assert_eq!(r.len(), 0);
    assert!(r.is_empty());
    assert_eq!(collect(r), Vec::<u8>::new());
  }

  #[test]
  fn left_only_basic() {
    let l = [1, 2, 3];
    let r = rb2(&l, &[]);
    assert_eq!(r.len(), 3);
    assert!(!r.is_empty());

    assert_eq!(r[0], 1);
    assert_eq!(r[1], 2);
    assert_eq!(r[2], 3);

    assert_eq!(r.get(0), Some(1));
    assert_eq!(r.get(2), Some(3));
    assert_eq!(r.get(3), None);

    let s = r.slice(0..3).unwrap();
    assert_eq!(collect(s), vec![1, 2, 3]);

    let s = r.slice(1..=2).unwrap();
    assert_eq!(collect(s), vec![2, 3]);

    let s = r.slice(..2).unwrap();
    assert_eq!(collect(s), vec![1, 2]);

    let s = r.slice(2..).unwrap();
    assert_eq!(collect(s), vec![3]);

    let s = r.slice(1..1).unwrap();
    assert!(s.is_empty());
    assert_eq!(collect(s), Vec::<u8>::new());
  }

  #[test]
  fn right_only_basic() {
    let rr = [7, 8, 9, 10];
    let r = rb2(&[], &rr);

    assert_eq!(r.len(), 4);
    assert_eq!(r[0], 7);
    assert_eq!(r[3], 10);
    assert_eq!(r.get(4), None);

    let s = r.slice(0..4).unwrap();
    assert_eq!(collect(s), vec![7, 8, 9, 10]);

    let s = r.slice(1..=2).unwrap();
    assert_eq!(collect(s), vec![8, 9]);

    let s = r.slice(..0).unwrap();
    assert!(s.is_empty());
    assert_eq!(collect(s), Vec::<u8>::new());
  }

  #[test]
  fn both_sides_iter_and_index() {
    let l = [1, 2];
    let rgt = [3, 4, 5];
    let r = rb2(&l, &rgt);
    assert_eq!(r.len(), 5);

    assert_eq!(collect(rb2(&l, &rgt)), vec![1, 2, 3, 4, 5]);

    assert_eq!(r[0], 1);
    assert_eq!(r[1], 2);
    assert_eq!(r[2], 3);
    assert_eq!(r[4], 5);
  }

  #[test]
  fn slice_within_left_right_and_crossing() {
    let l = [10, 11, 12];
    let rgt = [13, 14, 15, 16];
    let n0 = l.len();
    let r = rb2(&l, &rgt);

    let s = r.slice(1..3).unwrap();
    assert_eq!(collect(s), vec![11, 12]);

    let s = r.slice((n0 + 1)..(n0 + 3)).unwrap();
    assert_eq!(collect(s), vec![14, 15]);

    let s = r.slice(2..(n0 + 2)).unwrap();
    assert_eq!(collect(s), vec![12, 13, 14]);

    let s = r.slice(..).unwrap();
    assert_eq!(collect(s), vec![10, 11, 12, 13, 14, 15, 16]);
  }

  #[test]
  fn slice_inclusive_bounds_and_unbounded() {
    let l = [5, 6, 7];
    let rgt = [8, 9];
    let r = rb2(&l, &rgt);

    let s = r.slice(0..=1).unwrap();
    assert_eq!(collect(s), vec![5, 6]);

    let s = r.slice(2..=3).unwrap();
    assert_eq!(collect(s), vec![7, 8]);

    struct OpenClosed(usize, usize);
    impl RangeBounds<usize> for OpenClosed {
      fn start_bound(&self) -> Bound<&usize> {
        Bound::Excluded(&self.0)
      }
      fn end_bound(&self) -> Bound<&usize> {
        Bound::Included(&self.1)
      }
    }
    let s = r.slice(OpenClosed(1, 3)).unwrap();
    assert_eq!(collect(s), vec![7, 8]);

    let s = r.slice(..3).unwrap();
    assert_eq!(collect(s), vec![5, 6, 7]);

    let s = r.slice(..=3).unwrap();
    assert_eq!(collect(s), vec![5, 6, 7, 8]);

    let s = r.slice(3..).unwrap();
    assert_eq!(collect(s), vec![8, 9]);
  }

  #[test]
  fn slice_empty_exactly_at_split_should_be_empty() {
    let l = [1, 2, 3];
    let rgt = [4, 5];
    let n0 = l.len();
    let r = rb2(&l, &rgt);

    let s = r.slice(n0..n0).expect("Some(empty)");
    assert!(s.is_empty());
    assert_eq!(collect(s), Vec::<u8>::new());
  }

  #[test]
  fn slice_out_of_bounds_returns_none() {
    let l = [1, 2];
    let rgt = [3];
    let r = rb2(&l, &rgt);

    assert!(r.slice(0..10).is_none());
    assert!(r.slice(5..6).is_none());
    assert!(r.slice(3..2).is_none());
  }

  #[test]
  #[should_panic]
  fn index_out_of_bounds_panics() {
    let l = [1, 2];
    let rgt = [3];
    let r = rb2(&l, &rgt);
    let _ = r[3];
  }

  #[test]
  fn iterator_consumes_then_none() {
    let l = [42];
    let rgt = [43, 44];
    let mut r = rb2(&l, &rgt);

    assert_eq!(r.next(), Some(42));
    assert_eq!(r.next(), Some(43));
    assert_eq!(r.next(), Some(44));
    assert_eq!(r.next(), None);
    assert_eq!(r.next(), None);
  }

  #[test]
  fn get_matches_index_when_in_bounds() {
    let data_l = [9, 8];
    let data_r = [7, 6, 5];
    let r = rb2(&data_l, &data_r);
    for i in 0..r.len() {
      assert_eq!(r.get(i), Some(r[i]));
    }
    assert_eq!(r.get(r.len()), None);
  }

  #[test]
  fn full_chaining_equivalence() {
    let l = [10, 20, 30];
    let rgt = [40, 50];
    let r = rb2(&l, &rgt);

    let expected: Vec<u8> = l.iter().chain(rgt.iter()).copied().collect();
    let got = collect(r);
    assert_eq!(expected, got);
  }

  #[test]
  fn three_segment_basic() {
    type RB3<'a> = ReadbackReference<'a, 3>;
    let a: [u8; 2] = [1, 2];
    let b: [u8; 1] = [3];
    let c: [u8; 3] = [4, 5, 6];
    let r = RB3::from([&a as &[u8], &b as &[u8], &c as &[u8]]);

    assert_eq!(r.len(), 6);
    assert_eq!(collect(r), vec![1, 2, 3, 4, 5, 6]);

    // slice spanning b..c
    let s = r.slice(2..5).unwrap();
    assert_eq!(collect(s), vec![3, 4, 5]);
    // Ensure shape preserved: middle part contains 3..3, tail contains 4..5
    let parts = s.parts();
    assert_eq!(parts[0], &[]);
    assert_eq!(parts[1], &[3][..]);
    assert_eq!(parts[2], &[4, 5][..]);
  }
}
