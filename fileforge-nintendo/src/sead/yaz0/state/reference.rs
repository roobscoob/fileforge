use std::ops::{Bound, Index, RangeBounds};

pub struct ReadbackReference<'l>(&'l [u8], &'l [u8]);

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

impl<'l> ReadbackReference<'l> {
  #[inline]
  pub fn empty() -> Self {
    ReadbackReference(&[], &[])
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.0.len() + self.1.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  #[inline]
  pub fn get(&self, i: usize) -> Option<u8> {
    if i < self.0.len() {
      Some(self.0[i])
    } else {
      let j = i - self.0.len();
      self.1.get(j).copied()
    }
  }

  #[inline]
  pub fn slice<R: RangeBounds<usize>>(&self, r: R) -> Option<Self> {
    let (start, end) = normalize_bounds(r, self.len())?;
    assert!(start <= end, "invalid slice range");
    let n0 = self.0.len();

    Some(match (start < n0, end <= n0) {
      // entirely in left
      (true, true) => ReadbackReference(&self.0[start..end], &[]),
      // entirely in right
      (false, false) => {
        let s = start - n0;
        let e = end - n0;
        ReadbackReference(&self.1[s..e], &[])
      }
      // crosses split: left tail + right head
      (true, false) => {
        let left = &self.0[start..n0];
        let right = &self.1[..(end - n0)];
        ReadbackReference(left, right)
      }
      (false, true) => Self::empty(),
    })
  }
}

impl<'l> From<(&'l [u8], &'l [u8])> for ReadbackReference<'l> {
  fn from(value: (&'l [u8], &'l [u8])) -> Self {
    Self(value.0, value.1)
  }
}

impl<'l> Iterator for ReadbackReference<'l> {
  type Item = u8;

  fn next(&mut self) -> Option<Self::Item> {
    if self.0.len() != 0 {
      let (l, r) = self.0.split_first()?;
      self.0 = r;
      Some(*l)
    } else {
      let (l, r) = self.1.split_first()?;
      self.1 = r;
      Some(*l)
    }
  }
}

impl<'l> Index<usize> for ReadbackReference<'l> {
  type Output = u8;

  fn index(&self, i: usize) -> &Self::Output {
    let n0 = self.0.len();
    if i < n0 {
      &self.0[i]
    } else {
      &self.1[i - n0]
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use core::ops::RangeBounds;

  fn collect(rb: ReadbackReference<'_>) -> Vec<u8> {
    rb.into_iter().collect()
  }

  fn rb<'a>(l: &'a [u8], r: &'a [u8]) -> ReadbackReference<'a> {
    ReadbackReference::from((l, r))
  }

  #[test]
  fn empty_len_and_is_empty() {
    let r = ReadbackReference::empty();
    assert_eq!(r.len(), 0);
    assert!(r.is_empty());
    assert_eq!(collect(r), Vec::<u8>::new());
  }

  #[test]
  fn left_only_basic() {
    let l = [1, 2, 3];
    let r = rb(&l, &[]);
    assert_eq!(r.len(), 3);
    assert!(!r.is_empty());

    // indexing
    assert_eq!(r[0], 1);
    assert_eq!(r[1], 2);
    assert_eq!(r[2], 3);

    // get
    assert_eq!(r.get(0), Some(1));
    assert_eq!(r.get(2), Some(3));
    assert_eq!(r.get(3), None);

    // slice variants
    let s = r.slice(0..3).unwrap();
    assert_eq!(collect(s), vec![1, 2, 3]);

    let s = r.slice(1..=2).unwrap(); // inclusive end
    assert_eq!(collect(s), vec![2, 3]);

    let s = r.slice(..2).unwrap();
    assert_eq!(collect(s), vec![1, 2]);

    let s = r.slice(2..).unwrap();
    assert_eq!(collect(s), vec![3]);

    let s = r.slice(1..1).unwrap(); // empty range
    assert!(s.is_empty());
    assert_eq!(collect(s), Vec::<u8>::new());
  }

  #[test]
  fn right_only_basic() {
    let rr = [7, 8, 9, 10];
    let r = rb(&[], &rr);

    assert_eq!(r.len(), 4);
    assert_eq!(r[0], 7);
    assert_eq!(r[3], 10);
    assert_eq!(r.get(4), None);

    let s = r.slice(0..4).unwrap();
    assert_eq!(collect(s), vec![7, 8, 9, 10]);

    let s = r.slice(1..=2).unwrap();
    assert_eq!(collect(s), vec![8, 9]);

    let s = r.slice(..0).unwrap(); // empty
    assert!(s.is_empty());
    assert_eq!(collect(s), Vec::<u8>::new());
  }

  #[test]
  fn both_sides_iter_and_index() {
    let l = [1, 2];
    let rgt = [3, 4, 5];
    let r = rb(&l, &rgt);
    assert_eq!(r.len(), 5);

    // iteration should traverse left then right
    assert_eq!(collect(rb(&l, &rgt)), vec![1, 2, 3, 4, 5]);

    // indexing straddling boundary
    assert_eq!(r[0], 1);
    assert_eq!(r[1], 2);
    assert_eq!(r[2], 3); // first element of right
    assert_eq!(r[4], 5);
  }

  #[test]
  fn slice_within_left_right_and_crossing() {
    let l = [10, 11, 12];
    let rgt = [13, 14, 15, 16];
    let n0 = l.len();
    let r = rb(&l, &rgt);

    // fully in left
    let s = r.slice(1..3).unwrap();
    assert_eq!(collect(s), vec![11, 12]);

    // fully in right
    let s = r.slice((n0 + 1)..(n0 + 3)).unwrap();
    assert_eq!(collect(s), vec![14, 15]);

    // crossing boundary: tail of left + head of right
    let s = r.slice(2..(n0 + 2)).unwrap();
    assert_eq!(collect(s), vec![12, 13, 14]);

    // full slice
    let s = r.slice(..).unwrap();
    assert_eq!(collect(s), vec![10, 11, 12, 13, 14, 15, 16]);
  }

  #[test]
  fn slice_inclusive_bounds_and_unbounded() {
    let l = [5, 6, 7];
    let rgt = [8, 9];
    let r = rb(&l, &rgt);

    // inclusive end inside left
    let s = r.slice(0..=1).unwrap();
    assert_eq!(collect(s), vec![5, 6]);

    // inclusive end crossing boundary
    let s = r.slice(2..=3).unwrap();
    assert_eq!(collect(s), vec![7, 8]);

    // start excluded, end included: (1, 3] == 2..=3
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

    // .. (unbounded start)
    let s = r.slice(..3).unwrap();
    assert_eq!(collect(s), vec![5, 6, 7]);

    // ..= (unbounded start, inclusive end)
    let s = r.slice(..=3).unwrap();
    assert_eq!(collect(s), vec![5, 6, 7, 8]);

    // start.. (unbounded end)
    let s = r.slice(3..).unwrap();
    assert_eq!(collect(s), vec![8, 9]);
  }

  #[test]
  fn slice_empty_exactly_at_split_should_be_empty() {
    let l = [1, 2, 3];
    let rgt = [4, 5];
    let n0 = l.len();
    let r = rb(&l, &rgt);

    // Expect a valid empty view (both sides empty).
    let s = r.slice(n0..n0).expect("should yield Some(empty) for empty range at split");
    assert!(s.is_empty());
    assert_eq!(collect(s), Vec::<u8>::new());
  }

  #[test]
  fn slice_out_of_bounds_returns_none() {
    let l = [1, 2];
    let rgt = [3];
    let r = rb(&l, &rgt);

    assert!(r.slice(0..10).is_none()); // end > len
    assert!(r.slice(5..6).is_none()); // start > len
    assert!(r.slice(3..2).is_none()); // start > end rejected by normalize_bounds
  }

  #[test]
  #[should_panic]
  fn index_out_of_bounds_panics() {
    let l = [1, 2];
    let rgt = [3];
    let r = rb(&l, &rgt);
    let _ = r[3]; // len == 3; OOB
  }

  #[test]
  fn iterator_consumes_then_none() {
    let l = [42];
    let rgt = [43, 44];
    let mut r = rb(&l, &rgt);

    assert_eq!(r.next(), Some(42));
    assert_eq!(r.next(), Some(43));
    assert_eq!(r.next(), Some(44));
    assert_eq!(r.next(), None);
    assert_eq!(r.next(), None); // stays None
  }

  #[test]
  fn get_matches_index_when_in_bounds() {
    let data_l = [9, 8];
    let data_r = [7, 6, 5];
    let r = rb(&data_l, &data_r);
    for i in 0..r.len() {
      assert_eq!(r.get(i), Some(r[i]));
    }
    assert_eq!(r.get(r.len()), None);
  }

  #[test]
  fn full_chaining_equivalence() {
    let l = [10, 20, 30];
    let rgt = [40, 50];
    let r = rb(&l, &rgt);

    let expected: Vec<u8> = l.iter().chain(rgt.iter()).copied().collect();
    let got = collect(r);
    assert_eq!(expected, got);
  }
}
