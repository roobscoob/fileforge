pub mod builtins;
pub mod error;
pub mod hint;
pub mod r#ref;

use core::future::Future;

use error::{
  provider_mutate::ProviderMutateError, provider_read::ProviderReadError, provider_resize::ProviderResizeError, provider_slice::ProviderSliceError, user_mutate::UserMutateError,
  user_read::UserReadError, user_resize::UserResizeError, user_slice::UserSliceError,
};
use hint::ReadHint;

pub trait Provider<const NODE_NAME_SIZE: usize> {
  type StaticSliceProvider<'l, const SIZE: usize>: Provider<NODE_NAME_SIZE>
  where
    Self: 'l;

  type DynamicSliceProvider<'l>: Provider<NODE_NAME_SIZE>
  where
    Self: 'l;

  type SliceError: UserSliceError<NODE_NAME_SIZE>;
  type ReadError: UserReadError<NODE_NAME_SIZE>;

  fn len(&self) -> u64;

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>>;

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>>;

  fn read<const SIZE: usize, V, R: Future<Output = V>>(
    &self,
    offset: u64,
    hint: ReadHint,
    reader: impl for<'v> FnOnce(&'v [u8; SIZE]) -> R,
  ) -> impl Future<Output = Result<V, ProviderReadError<NODE_NAME_SIZE, Self::ReadError>>>;
}

pub trait MutProvider<const NODE_NAME_SIZE: usize>: Provider<NODE_NAME_SIZE> {
  type MutateError: UserMutateError<NODE_NAME_SIZE>;

  type StaticMutSliceProvider<'l, const SIZE: usize>: Provider<NODE_NAME_SIZE>
  where
    Self: 'l;

  type DynamicMutSliceProvider<'l>: Provider<NODE_NAME_SIZE>
  where
    Self: 'l;

  fn mutate<const SIZE: usize, V, R: Future<Output = V>>(
    &mut self,
    offset: u64,
    writer: impl for<'v> FnOnce(&'v mut [u8; SIZE]) -> R,
  ) -> impl Future<Output = Result<V, ProviderMutateError<NODE_NAME_SIZE, Self::MutateError>>>;

  fn mut_slice<'l, const SIZE: usize>(&'l mut self, start: u64) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>>;

  fn mut_slice_dynamic<'l>(&'l mut self, start: u64, size: Option<u64>) -> Result<Self::DynamicMutSliceProvider<'l>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>>;
}

pub trait ResizableProvider<const NODE_NAME_SIZE: usize>: MutProvider<NODE_NAME_SIZE> {
  type ResizeError: UserResizeError<NODE_NAME_SIZE>;

  fn resize_at(&mut self, offset: u64, old_len: u64, new_len: u64) -> impl Future<Output = Result<(), ProviderResizeError<NODE_NAME_SIZE, Self::ResizeError>>>;
}
