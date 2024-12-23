use crate::{
  provider::out_of_bounds::SliceOutOfBoundsError,
  reader::error::underlying_provider_stat::UnderlyingProviderStatError,
};

use super::ProviderError;

pub enum SliceError<Se: ProviderError> {
  OutOfBounds(SliceOutOfBoundsError),
  StatError(UnderlyingProviderStatError<Se>),
}

impl<Se: ProviderError> From<SliceOutOfBoundsError> for SliceError<Se> {
  fn from(value: SliceOutOfBoundsError) -> Self { SliceError::OutOfBounds(value) }
}

impl<Se: ProviderError> From<UnderlyingProviderStatError<Se>> for SliceError<Se> {
  fn from(value: UnderlyingProviderStatError<Se>) -> Self { SliceError::StatError(value) }
}
