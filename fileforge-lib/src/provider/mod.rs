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

pub trait Provider {
  type StaticSliceProvider<'l, const SIZE: usize>: Provider
  where
    Self: 'l;

  type DynamicSliceProvider<'l>: Provider
  where
    Self: 'l;

  type SliceError: UserSliceError;
  type ReadError: UserReadError;

  fn len(&self) -> u64;

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>>;

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<Self::SliceError>>;

  async fn read<const SIZE: usize, V>(
    &self,
    offset: u64,
    hint: ReadHint,
    reader: impl for<'v> AsyncFnOnce(&'v [u8; SIZE]) -> V,
  ) -> Result<V, ProviderReadError<Self::ReadError>>;
}

pub trait MutProvider: Provider {
  type MutateError: UserMutateError;

  type StaticMutSliceProvider<'l, const SIZE: usize>: Provider
  where
    Self: 'l;

  type DynamicMutSliceProvider<'l>: Provider
  where
    Self: 'l;

  async fn mutate<const SIZE: usize, V>(
    &mut self,
    offset: u64,
    writer: impl for<'v> AsyncFnOnce(&'v mut [u8; SIZE]) -> V,
  ) -> Result<V, ProviderMutateError<Self::MutateError>>;

  fn mut_slice<'l, const SIZE: usize>(&'l mut self, start: u64) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>>;

  fn mut_slice_dynamic<'l>(&'l mut self, start: u64, size: Option<u64>) -> Result<Self::DynamicMutSliceProvider<'l>, ProviderSliceError<Self::SliceError>>;
}

pub trait ResizableProvider: MutProvider {
  type ResizeError: UserResizeError;

  async fn resize_at(&mut self, offset: u64, old_len: u64, new_len: u64) -> Result<(), ProviderResizeError<Self::ResizeError>>;
}
