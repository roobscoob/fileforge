use core::cmp::Ordering;

pub fn fallible_mapping_binary_search<T, E>(
  length: usize,
  mut map: impl FnMut(usize) -> Result<T, E>,
  mut compare: impl FnMut(&T) -> Ordering,
) -> Result<Option<T>, E> {
  let mut low = 0;
  let mut high = length - 1;

  while low <= high {
    let mid = low + (high - low) / 2;
    let mapped = map(mid)?;

    match compare(&mapped) {
      Ordering::Equal => return Ok(Some(mapped)),
      Ordering::Greater => low = mid + 1,
      Ordering::Less => {
        if mid == low {
          return Ok(None);
        }

        high = mid
      }
    }
  }

  Ok(None)
}

pub fn fallible_binary_search<E>(
  length: usize,
  mut compare: impl FnMut(usize) -> Result<Ordering, E>,
) -> Result<Option<usize>, E> {
  let mut low = 0;
  let mut high = length - 1;

  while low <= high {
    let mid = low + (high - low) / 2;

    match compare(mid)? {
      Ordering::Equal => return Ok(Some(mid)),
      Ordering::Greater => low = mid + 1,
      Ordering::Less => {
        if mid == low {
          return Ok(None);
        }

        high = mid
      }
    }
  }

  Ok(None)
}

pub fn binary_search(length: usize, mut compare: impl FnMut(usize) -> Ordering) -> Option<usize> {
  let mut low = 0;
  let mut high = length - 1;

  while low <= high {
    let mid = low + (high - low) / 2;

    match compare(mid) {
      Ordering::Equal => return Some(mid),
      Ordering::Greater => low = mid + 1,
      Ordering::Less => {
        if mid == low {
          return None;
        }

        high = mid
      }
    }
  }

  None
}
