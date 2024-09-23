use crate::{diagnostic::node::reference::DiagnosticReference, provider::error::{read_error::ReadError, ProviderError}};

use super::{out_of_bounds::ReadOutOfBoundsError, underlying_provider_read::UnderlyingProviderReadError};

pub enum ParsePrimitiveError<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  OutOfBounds(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderReadError(UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
}

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  pub fn from_read_error(value: ReadError<Re>, location: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::UnderlyingProviderReadError(UnderlyingProviderReadError(value.0, location))
  }
}

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> From<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>> for ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn from(value: ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::OutOfBounds(value)
  }
}

pub trait ParsePrimitiveErrorResultExtension<'pool, Success, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  fn map_out_of_bounds<Ne, Cb: FnOnce(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Ne>(self, cb: Cb) -> Result<Result<Success, UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne>;
}

impl<'pool, Success, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> ParsePrimitiveErrorResultExtension<'pool, Success, Re, DIAGNOSTIC_NODE_NAME_SIZE> for Result<Success, ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>> {
  fn map_out_of_bounds<Ne, Cb: FnOnce(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Ne>(self, cb: Cb) -> Result<Result<Success, UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne> {
    match self {
      Ok(v) => Ok(Ok(v)),
      Err(ParsePrimitiveError::UnderlyingProviderReadError(upre)) => Ok(Err(upre)),
      Err(ParsePrimitiveError::OutOfBounds(oob)) => Err(cb(oob)),
    }
  }
}