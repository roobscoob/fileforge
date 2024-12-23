use crate::{
  diagnostic::node::reference::DiagnosticReference,
  error::{report::Report, Error},
  provider::error::{read_error::ReadError, ProviderError},
};

use super::{
  out_of_bounds::ReadOutOfBoundsError, underlying_provider_error::UnderlyingProviderError,
  underlying_provider_read::UnderlyingProviderReadError,
  underlying_provider_stat::UnderlyingProviderStatError,
};

pub enum ParsePrimitiveError<
  'pool,
  Re: ProviderError,
  Se: ProviderError,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
> {
  OutOfBounds(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderReadError(UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderStatError(UnderlyingProviderStatError<Se>),
}

impl<'pool, Re: ProviderError, Se: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  pub fn from_read_error(
    value: ReadError<Re>,
    location: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Self {
    Self::UnderlyingProviderReadError(UnderlyingProviderReadError(value.0, location))
  }
}

impl<'pool, Re: ProviderError, Se: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  From<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>
  for ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn from(value: ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::OutOfBounds(value)
  }
}

pub trait ParsePrimitiveErrorResultExtension<
  'pool,
  Success,
  Re: ProviderError,
  Se: ProviderError,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
>
{
  fn map_out_of_bounds<
    Ne,
    Cb: FnOnce(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Ne,
  >(
    self,
    cb: Cb,
  ) -> Result<Result<Success, UnderlyingProviderError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne>;
}

impl<
    'pool,
    Success,
    Re: ProviderError,
    Se: ProviderError,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > ParsePrimitiveErrorResultExtension<'pool, Success, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
  for Result<Success, ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>
{
  fn map_out_of_bounds<
    Ne,
    Cb: FnOnce(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Ne,
  >(
    self,
    cb: Cb,
  ) -> Result<Result<Success, UnderlyingProviderError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne>
  {
    match self {
      Ok(v) => Ok(Ok(v)),
      Err(ParsePrimitiveError::UnderlyingProviderReadError(upre)) => {
        Ok(Err(UnderlyingProviderError::ReadError(upre)))
      }
      Err(ParsePrimitiveError::UnderlyingProviderStatError(upse)) => {
        Ok(Err(UnderlyingProviderError::StatError(upse)))
      }
      Err(ParsePrimitiveError::OutOfBounds(oob)) => Err(cb(oob)),
    }
  }
}

impl<'pool, Re: ProviderError, Se: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  ProviderError for ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<
    'p,
    Cb: FnMut(crate::error::report::Report<NODE_NAME_SIZE>) -> (),
    const NODE_NAME_SIZE: usize,
  >(
    &self,
    location: Option<DiagnosticReference<'p, NODE_NAME_SIZE>>,
    callback: Cb,
  ) {
    unimplemented!()
  }
}

impl<'pool, Re: ProviderError, Se: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      Self::OutOfBounds(oob) => oob.with_report(callback),
      Self::UnderlyingProviderReadError(UnderlyingProviderReadError(ure, location)) => {
        ure.with_report(Some(*location), callback)
      }
      Self::UnderlyingProviderStatError(UnderlyingProviderStatError(ure)) => {
        ure.with_report(None, callback)
      }
    }
  }
}
