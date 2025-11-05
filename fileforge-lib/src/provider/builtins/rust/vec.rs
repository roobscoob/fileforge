use crate::provider::{
  builtins::slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider},
  error::{out_of_bounds::OutOfBoundsError, provider_read::ProviderReadError, provider_slice::ProviderSliceError},
  hint::ReadHint,
  MutProvider, Provider, ResizableProvider,
};

impl<T> Provider for alloc::vec::Vec<T>
where
  T: Copy,
{
  type Type = T;

  type ReadError = core::convert::Infallible;
  type SliceError = core::convert::Infallible;

  type StaticSliceProvider<'a, const SIZE: usize>
    = &'a [T; SIZE]
  where
    T: 'a;

  type DynamicSliceProvider<'l>
    = DynamicSliceProvider<&'l alloc::vec::Vec<T>>
  where
    Self: 'l;

  fn len(&self) -> u64 {
    self.len() as u64
  }

  async fn read<const SIZE: usize, V>(&self, offset: u64, _hint: ReadHint, reader: impl AsyncFnOnce(&[T; SIZE]) -> V) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(self.len() as u64, offset, Some(SIZE as u64))?;

    Ok(reader(&self[offset as usize..(offset + SIZE as u64) as usize].split_first_chunk::<SIZE>().unwrap().0).await)
  }

  fn slice<'a, const SIZE: usize>(&'a self, start: u64) -> Result<Self::StaticSliceProvider<'a, SIZE>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, Some(SIZE as u64))?;

    let slice = &self[start as usize..(start + SIZE as u64) as usize];
    let slice: &[T; SIZE] = slice.try_into().unwrap();

    Ok(slice)
  }

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl<T> MutProvider for alloc::vec::Vec<T>
where
  T: Copy,
{
  type MutateError = core::convert::Infallible;

  type DynamicMutSliceProvider<'l>
    = DynamicSliceProvider<&'l mut alloc::vec::Vec<T>>
  where
    Self: 'l;

  type StaticMutSliceProvider<'l, const SIZE: usize>
    = FixedSliceProvider<SIZE, &'l mut alloc::vec::Vec<T>>
  where
    Self: 'l;

  async fn mutate<const SIZE: usize, V>(
    &mut self,
    offset: u64,
    writer: impl AsyncFnOnce(&mut [Self::Type; SIZE]) -> V,
  ) -> Result<V, crate::provider::error::provider_mutate::ProviderMutateError<Self::MutateError>> {
    OutOfBoundsError::assert(<&mut alloc::vec::Vec<T> as Provider>::len(&self) as u64, offset, Some(SIZE as u64))?;

    Ok(writer(&mut self[offset as usize..(offset + SIZE as u64) as usize].split_first_chunk_mut::<SIZE>().unwrap().0).await)
  }

  fn mut_slice<'l, const SIZE: usize>(&'l mut self, start: u64) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> {
    Ok(FixedSliceProvider::new(start, self)?)
  }

  fn mut_slice_dynamic<'l>(&'l mut self, start: u64, size: Option<u64>) -> Result<Self::DynamicMutSliceProvider<'l>, ProviderSliceError<Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl ResizableProvider for alloc::vec::Vec<u8> {
  type ResizeError = core::convert::Infallible;

  async fn resize_at(&mut self, offset: u64, old_len: u64, new_len: u64) -> Result<(), crate::provider::error::provider_resize::ProviderResizeError<Self::ResizeError>> {
    // Validate region exists in the current buffer.
    OutOfBoundsError::assert(<&mut alloc::vec::Vec<u8> as Provider>::len(&self) as u64, offset, Some(old_len))?;

    if new_len == old_len {
      return Ok(());
    }

    let old_total = <&mut alloc::vec::Vec<u8> as Provider>::len(&self) as usize;
    let old_region_end = (offset + old_len) as usize; // start of tail (before growth/shrink)
    let tail_len = old_total - old_region_end;

    if new_len > old_len {
      // GROW: make room inside the vector, then slide the tail to the right.
      let grow_by = (new_len - old_len) as usize;
      let new_total = old_total + grow_by;

      // 1) extend the vec at the end so there's space for the tail to move into
      self.resize(new_total, 0);

      // 2) move the tail (which was [old_region_end .. old_total]) to the right by grow_by
      let src_start = old_region_end;
      let src_end = old_region_end + tail_len; // == old_total
      let dst_start = old_region_end + grow_by;
      self.copy_within(src_start..src_end, dst_start);

      // 3) zero-fill the newly inserted gap inside the region
      self[old_region_end..old_region_end + grow_by].fill(0);
    } else {
      // SHRINK: slide the tail left over the removed bytes, then truncate.
      let shrink_by = (old_len - new_len) as usize;
      let new_total = old_total - shrink_by;

      // move the tail (which currently starts at old_region_end) left by shrink_by
      let src_start = old_region_end;
      let src_end = old_total;
      let dst_start = old_region_end - shrink_by;
      self.copy_within(src_start..src_end, dst_start);

      // finally trim the now-duplicated tail at the end
      self.truncate(new_total);
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::provider::{
    builtins::slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider},
    error::{provider_mutate::ProviderMutateError, provider_read::ProviderReadError, provider_resize::ProviderResizeError, provider_slice::ProviderSliceError},
    hint::ReadHint,
    MutProvider, Provider, ResizableProvider,
  };

  // A helper to produce a default ReadHint without assuming a specific variant.
  fn hint() -> ReadHint {
    ReadHint::new()
  }

  // ---------- Provider::len -------------------------------------------------

  #[test]
  fn provider_len_matches_vec_len() {
    let v = vec![1u8, 2, 3, 4, 5];
    assert_eq!(<Vec<u8> as Provider>::len(&v), 5);
    assert_eq!(v.len() as u64, <Vec<u8> as Provider>::len(&v));
  }

  // ---------- Provider::read (in-bounds / out-of-bounds) -------------------

  #[tokio::test]
  async fn provider_read_in_bounds() {
    let v = vec![10u8, 11, 12, 13, 14, 15];
    // Read 3 bytes starting at offset 2 → [12,13,14]
    let got = <Vec<u8> as Provider>::read::<3, _>(&v, 2, hint(), async move |chunk| {
      // Copy out for assertion
      [chunk[0], chunk[1], chunk[2]]
    })
    .await
    .expect("read should succeed");
    assert_eq!(got, [12, 13, 14]);
  }

  #[tokio::test]
  async fn provider_read_exact_end_boundary() {
    let v = vec![0u8, 1, 2, 3];
    // Read last 2 bytes at offsets 2..4
    let got = <Vec<u8> as Provider>::read::<2, _>(&v, 2, hint(), async move |chunk| [chunk[0], chunk[1]])
      .await
      .expect("read should succeed at end boundary");
    assert_eq!(got, [2, 3]);
  }

  #[tokio::test]
  async fn provider_read_out_of_bounds() {
    let v = vec![10u8, 11, 12, 13];
    // Ask for 3 bytes starting at offset 3 (needs 3..6) → OOB
    let err = <Vec<u8> as Provider>::read::<3, _>(&v, 3, hint(), async move |_chunk| {}).await.expect_err("read must fail");
    // Be precise if the enum exposes an OutOfBounds variant; otherwise just assert error.
    let is_oob = matches!(err, ProviderReadError::OutOfBounds(_));
    assert!(is_oob, "expected ProviderReadError::OutOfBounds, got {:?}", err);
  }

  // ---------- FixedSliceProvider (read) ------------------------------------

  #[tokio::test]
  async fn fixed_slice_provider_read() {
    let v = vec![1u8, 2, 3, 4, 5, 6];
    // Fixed window of size 3 starting at index 2 → covers [3,4,5]
    let slice = <Vec<u8> as Provider>::slice::<3>(&v, 2).expect("slice ok");
    // Reading inside the slice at local offset 0 should give exactly [3,4,5].
    let got = Provider::read::<3, _>(&slice, 0, hint(), async move |chunk| [chunk[0], chunk[1], chunk[2]]).await.expect("read ok");
    assert_eq!(got, [3, 4, 5]);
  }

  #[test]
  fn fixed_slice_provider_rejects_oob_start() {
    let v = vec![1u8, 2, 3];
    // size=4 starting at 0 is OOB
    let err = <Vec<u8> as Provider>::slice::<4>(&v, 0).map(|_| {}).expect_err("slice must fail");
    let is_oob = matches!(err, ProviderSliceError::OutOfBounds(_));
    assert!(is_oob, "expected ProviderSliceError::OutOfBounds, got {:?}", err);
  }

  // ---------- DynamicSliceProvider (read) ----------------------------------

  #[tokio::test]
  async fn dynamic_slice_provider_read_sized() {
    let v = vec![10u8, 11, 12, 13, 14, 15];
    // Dynamic slice of length 4 starting at 1 → [11,12,13,14]
    let ds: DynamicSliceProvider<&_> = <Vec<u8> as Provider>::slice_dynamic(&v, 1, Some(4)).expect("slice ok");

    // Read first 2 bytes from the dynamic slice (local offset 0)
    let got = <DynamicSliceProvider<&_> as Provider>::read::<2, _>(&ds, 0, hint(), async move |chunk| [chunk[0], chunk[1]])
      .await
      .expect("read ok");
    assert_eq!(got, [11, 12]);

    // Read next 2 bytes (local offset 2)
    let got2 = <DynamicSliceProvider<&_> as Provider>::read::<2, _>(&ds, 2, hint(), async move |chunk| [chunk[0], chunk[1]])
      .await
      .expect("read ok");
    assert_eq!(got2, [13, 14]);
  }

  #[test]
  fn dynamic_slice_provider_rejects_oob_window() {
    let v = vec![1u8, 2, 3, 4];
    // Start at 3, size 3 → would require 3..6, OOB
    let err = <Vec<u8> as Provider>::slice_dynamic(&v, 3, Some(3)).map(|_| {}).expect_err("slice must fail");
    let is_oob = matches!(err, ProviderSliceError::OutOfBounds(_));
    assert!(is_oob, "expected ProviderSliceError::OutOfBounds, got {:?}", err);
  }

  // ---------- MutProvider::mutate (direct) ---------------------------------

  #[tokio::test]
  async fn mutate_in_place_writes_back() {
    let mut v = vec![0u8, 1, 2, 3, 4, 5];
    // Overwrite 2 bytes at offset 2 with [9, 8].
    let res = <Vec<u8> as MutProvider>::mutate::<2, _>(&mut v, 2, async move |chunk| {
      chunk[0] = 9;
      chunk[1] = 8;
      // Return something just to ensure closure result is forwarded.
      (chunk[0], chunk[1])
    })
    .await
    .expect("mutate ok");
    assert_eq!(res, (9, 8));
    assert_eq!(v, vec![0, 1, 9, 8, 4, 5]);
  }

  #[tokio::test]
  async fn mutate_out_of_bounds_fails() {
    let mut v = vec![1u8, 2, 3, 4];
    // Need 3 bytes at offset 3 → OOB
    let err = <Vec<u8> as MutProvider>::mutate::<3, _>(&mut v, 3, async move |_chunk| ()).await.expect_err("mutate must fail");
    let is_oob = matches!(err, ProviderMutateError::OutOfBounds(_));
    assert!(is_oob, "expected ProviderMutateError::OutOfBounds, got {:?}", err);
  }

  // ---------- mut_slice / StaticMutSliceProvider ---------------------------

  #[tokio::test]
  async fn mut_slice_fixed_provider_mutates_underlying() {
    let mut v = vec![10u8, 20, 30, 40, 50];
    // Fixed mutable slice of 2 at start=2 → targets [30,40]
    let mut s: FixedSliceProvider<2, &mut Vec<u8>> = <Vec<u8> as MutProvider>::mut_slice::<2>(&mut v, 2).expect("mut_slice ok");

    // Read before mutating
    let before = <FixedSliceProvider<2, &mut Vec<u8>> as Provider>::read::<2, _>(&s, 0, hint(), async move |c| [c[0], c[1]])
      .await
      .expect("read ok");
    assert_eq!(before, [30, 40]);

    // Mutate in place
    <FixedSliceProvider<2, &mut Vec<u8>> as MutProvider>::mutate::<2, _>(&mut s, 0, async move |chunk| {
      chunk[0] = 77;
      chunk[1] = 88;
    })
    .await
    .expect("mutate ok");

    // Underlying vec reflects changes
    assert_eq!(v, vec![10, 20, 77, 88, 50]);
  }

  // ---------- mut_slice_dynamic / DynamicMutSliceProvider ------------------

  #[tokio::test]
  async fn mut_slice_dynamic_provider_mutates_underlying() {
    let mut v = vec![1u8, 2, 3, 4, 5, 6, 7];
    // Dynamic mutable slice (size 3) at start=3 → targets [4,5,6]
    let mut ds: DynamicSliceProvider<&mut Vec<u8>> = <Vec<u8> as MutProvider>::mut_slice_dynamic(&mut v, 3, Some(3)).expect("mut_slice_dynamic ok");

    // Read first 2 bytes locally
    let got = <DynamicSliceProvider<&mut Vec<u8>> as Provider>::read::<2, _>(&ds, 0, hint(), async move |c| [c[0], c[1]])
      .await
      .expect("read ok");
    assert_eq!(got, [4, 5]);

    // Mutate last 2 bytes locally (offset 1..3)
    <DynamicSliceProvider<&mut Vec<u8>> as MutProvider>::mutate::<2, _>(&mut ds, 1, async move |c| {
      c[0] = 90; // was 5
      c[1] = 91; // was 6
    })
    .await
    .expect("mutate ok");

    // Underlying becomes [1,2,3, 4,90,91, 7]
    assert_eq!(v, vec![1, 2, 3, 4, 90, 91, 7]);
  }

  // ---------- ResizableProvider::resize_at (Vec<u8>) -----------------------

  #[tokio::test]
  async fn resize_at_noop_when_lengths_equal() {
    let mut v = b"ABCDE".to_vec();
    <Vec<u8> as ResizableProvider>::resize_at(&mut v, 2, 3, 3).await.expect("noop ok");
    assert_eq!(v, b"ABCDE");
  }

  #[tokio::test]
  async fn resize_at_grow_in_middle_inserts_zeros_and_shifts_tail() {
    // Layout: [0,1,2,  10,11,  3,4,5,6]
    let mut v = vec![0u8, 1, 2, 10, 11, 3, 4, 5, 6];
    // Grow region starting at offset=3 (old_len=2 → new_len=5). Insert 3 zero bytes after [10,11].
    <Vec<u8> as ResizableProvider>::resize_at(&mut v, 3, 2, 5).await.expect("grow ok");
    // Expect: [0,1,2, 10,11, 0,0,0, 3,4,5,6]
    assert_eq!(v, vec![0, 1, 2, 10, 11, 0, 0, 0, 3, 4, 5, 6]);
  }

  #[tokio::test]
  async fn resize_at_shrink_in_middle_removes_bytes_and_shifts_tail_left() {
    // Layout: [0,1,2,  10,11,12,13,14,  3,4,5]
    let mut v = vec![0u8, 1, 2, 10, 11, 12, 13, 14, 3, 4, 5];
    // Shrink region at offset=3 from old_len=5 to new_len=2 (remove 3 bytes 12,13,14)
    <Vec<u8> as ResizableProvider>::resize_at(&mut v, 3, 5, 2).await.expect("shrink ok");
    // Expect: [0,1,2, 10,11, 3,4,5]
    assert_eq!(v, vec![0, 1, 2, 10, 11, 3, 4, 5]);
  }

  #[tokio::test]
  async fn resize_at_grow_at_start() {
    let mut v = vec![9u8, 8, 7, 6];
    // Region is first 1 byte -> grow to 4 bytes, insert 3 zeros after original first byte
    <Vec<u8> as ResizableProvider>::resize_at(&mut v, 0, 1, 4).await.expect("grow start ok");
    assert_eq!(v, vec![9, 0, 0, 0, 8, 7, 6]);
  }

  #[tokio::test]
  async fn resize_at_shrink_at_start() {
    let mut v = vec![1u8, 2, 3, 4, 5];
    // Region is first 4 bytes -> shrink to 2: remove two bytes at the end of region
    <Vec<u8> as ResizableProvider>::resize_at(&mut v, 0, 4, 2).await.expect("shrink start ok");
    assert_eq!(v, vec![1, 2, 5]);
  }

  #[tokio::test]
  async fn resize_at_grow_at_end() {
    let mut v = vec![1u8, 2, 3, 4];
    // Region is the last 1 byte (offset=3), grow to 3 (insert two zeros after it)
    <Vec<u8> as ResizableProvider>::resize_at(&mut v, 3, 1, 3).await.expect("grow end ok");
    assert_eq!(v, vec![1, 2, 3, 4, 0, 0]);
  }

  #[tokio::test]
  async fn resize_at_shrink_at_end() {
    let mut v = vec![1u8, 2, 3, 4, 5, 6];
    // Region is last 4 bytes (offset=2), shrink to 1 (remove 3 bytes)
    <Vec<u8> as ResizableProvider>::resize_at(&mut v, 2, 4, 1).await.expect("shrink end ok");
    assert_eq!(v, vec![1, 2, 3]);
  }

  #[tokio::test]
  async fn resize_at_rejects_oob_region() {
    let mut v = vec![1u8, 2, 3, 4];
    // Region (offset=3, old_len=2) would require [3..5], OOB
    let err = <Vec<u8> as ResizableProvider>::resize_at(&mut v, 3, 2, 4).await.expect_err("resize must fail");
    let is_oob = matches!(err, ProviderResizeError::OutOfBounds(_));
    assert!(is_oob, "expected ProviderResizeError::OutOfBounds, got {:?}", err);
  }

  // ---------- Round-trips across APIs --------------------------------------

  #[tokio::test]
  async fn roundtrip_slice_read_then_mutate_then_resize() {
    // Start with a clean sequence
    let mut v = (0u8..=9).collect::<Vec<_>>(); // [0,1,2,3,4,5,6,7,8,9]

    // 1) Read a slice [3..7) via dynamic slice (size 4), ensure content is [3,4,5,6].
    let ds: DynamicSliceProvider<&Vec<u8>> = <Vec<u8> as Provider>::slice_dynamic(&v, 3, Some(4)).expect("slice ok");
    let part = <DynamicSliceProvider<&Vec<u8>> as Provider>::read::<4, _>(&ds, 0, hint(), async move |c| [c[0], c[1], c[2], c[3]])
      .await
      .expect("read ok");
    assert_eq!(part, [3, 4, 5, 6]);

    // 2) Mutate the middle two bytes of that region to 100,101 (local offset 1..3).
    let mut dsm: DynamicSliceProvider<&mut Vec<u8>> = <Vec<u8> as MutProvider>::mut_slice_dynamic(&mut v, 3, Some(4)).expect("mut_slice_dynamic ok");
    <DynamicSliceProvider<&mut Vec<u8>> as MutProvider>::mutate::<2, _>(&mut dsm, 1, async move |c| {
      c[0] = 100; // replaces 4
      c[1] = 101; // replaces 5
    })
    .await
    .expect("mutate ok");

    assert_eq!(v, vec![0, 1, 2, 3, 100, 101, 6, 7, 8, 9]);

    // 3) Resize that region (start=3, old_len=4 -> new_len=6): insert two zeros after it.
    <Vec<u8> as ResizableProvider>::resize_at(&mut v, 3, 4, 6).await.expect("resize ok");
    // Expected: [0,1,2, 3,100,101,6, 0,0, 7,8,9]
    assert_eq!(v, vec![0, 1, 2, 3, 100, 101, 6, 0, 0, 7, 8, 9]);
  }
}
