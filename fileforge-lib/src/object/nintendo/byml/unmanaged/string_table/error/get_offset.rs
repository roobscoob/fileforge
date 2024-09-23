use crate::{diagnostic::node::reference::DiagnosticReference, provider::r#trait::Provider, reader::error::underlying_provider_read::UnderlyingProviderReadError};

use super::get_length::StringTableNotLargeEnough;

pub enum GetOffsetError<'pool, P: Provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  UnderlyingProviderError(UnderlyingProviderReadError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>),
  IndexOutOfBounds { requested_index: u32, length_dr: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>, length_value: u32 },
  NotLargeEnough(StringTableNotLargeEnough),
}