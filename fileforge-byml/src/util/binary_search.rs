use core::cmp::Ordering;

pub fn fallible_binary_search<E>(
  length: usize,
  mut compare: impl FnMut(usize) -> Result<Ordering, E>,
) -> Result<Option<usize>, E> {
  let mut low = 0;
  let mut high = length;

  while low <= high {
    let mid = low + (high - low) / 2;

    match compare(mid)? {
      Ordering::Equal => return Ok(Some(mid)),
      Ordering::Greater => low = mid + 1,
      Ordering::Less => high = mid,
    }
  }

  Ok(None)
}

pub fn binary_search(length: usize, mut compare: impl FnMut(usize) -> Ordering) -> Option<usize> {
  let mut low = 0;
  let mut high = length;

  while low <= high {
    let mid = low + (high - low) / 2;

    match compare(mid) {
      Ordering::Equal => return Some(mid),
      Ordering::Greater => low = mid + 1,
      Ordering::Less => high = mid,
    }
  }

  None
}
