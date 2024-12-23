use crate::provider::error::ProviderError;

use super::underlying_provider_stat::UnderlyingProviderStatError;

pub enum SeekError<Se: ProviderError> {
  Overflowed { available_size: u64 },
  UnderlyingProviderStatError(UnderlyingProviderStatError<Se>),
}
