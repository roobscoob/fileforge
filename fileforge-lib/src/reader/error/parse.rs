use crate::{
  diagnostic::node::reference::DiagnosticReference,
  error::{report::Report, Error},
  provider::error::{never::Never, read_error::ReadError, slice_error::SliceError, ProviderError},
};

use super::{
  domain::DomainError, out_of_bounds::ReadOutOfBoundsError, parse_primitive::ParsePrimitiveError,
  underlying_provider_read::UnderlyingProviderReadError,
  underlying_provider_stat::UnderlyingProviderStatError,
};

pub enum ParseError<
  'pool,
  T: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
  Re: ProviderError,
  Se: ProviderError,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
> {
  OutOfBounds(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderReadError(UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderStatError(UnderlyingProviderStatError<Se>),
  DomainSpecific(T),
}

pub trait IntoParseResult<
  'pool,
  T,
  DSE: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
  Re: ProviderError,
  Se: ProviderError,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
>
{
  fn into_parse_result<
    GetDiagnosticRef: FnOnce() -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  >(
    self,
    get_diagnostic_ref: GetDiagnosticRef,
  ) -> Result<T, ParseError<'pool, DSE, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>;
}

impl<
    'pool,
    T,
    DSE: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
    Re: ProviderError,
    Se: ProviderError,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > IntoParseResult<'pool, T, DSE, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
  for Result<T, SliceError<Se>>
{
  fn into_parse_result<
    GetDiagnosticRef: FnOnce() -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  >(
    self,
    get_diagnostic_ref: GetDiagnosticRef,
  ) -> Result<T, ParseError<'pool, DSE, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>> {
    self.map_err(|e| match e {
      SliceError::OutOfBounds(oob) => ParseError::OutOfBounds(
        ReadOutOfBoundsError::from_slice_out_of_bounds_error(oob, get_diagnostic_ref()),
      ),
      SliceError::StatError(se) => ParseError::UnderlyingProviderStatError(se),
    })
  }
}

pub trait ParseErrorResultExtension<
  'pool,
  Success,
  T: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
  Re: ProviderError,
  Se: ProviderError,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
>
{
  fn map_domain_specific<Ne, Cb: FnOnce(T) -> Ne>(
    self,
    cb: Cb,
  ) -> Result<Result<Success, ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne>;
}

impl<
    'pool,
    Success,
    T: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
    Re: ProviderError,
    Se: ProviderError,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > ParseErrorResultExtension<'pool, Success, T, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
  for Result<Success, ParseError<'pool, T, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>
{
  fn map_domain_specific<Ne, Cb: FnOnce(T) -> Ne>(
    self,
    cb: Cb,
  ) -> Result<Result<Success, ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne>
  {
    match self {
      Ok(v) => Ok(Ok(v)),
      Err(ParseError::DomainSpecific(e)) => Err(cb(e)),
      Err(ParseError::OutOfBounds(oob)) => Ok(Err(ParsePrimitiveError::OutOfBounds(oob))),
      Err(ParseError::UnderlyingProviderStatError(upse)) => {
        Ok(Err(ParsePrimitiveError::UnderlyingProviderStatError(upse)))
      }
      Err(ParseError::UnderlyingProviderReadError(upre)) => {
        Ok(Err(ParsePrimitiveError::UnderlyingProviderReadError(upre)))
      }
    }
  }
}

impl<
    'pool,
    T: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
    Re: ProviderError,
    Se: ProviderError,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > ParseError<'pool, T, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  pub fn domain_err(value: T) -> Self {
    ParseError::<T, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>::DomainSpecific(value)
  }

  pub fn from_read_error(
    value: ReadError<Re>,
    location: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Self {
    Self::UnderlyingProviderReadError(UnderlyingProviderReadError(value.0, location))
  }

  pub fn map_domains<N: Error<DIAGNOSTIC_NODE_NAME_SIZE>, M: FnOnce(T) -> N>(
    self,
    mapper: M,
  ) -> ParseError<'pool, N, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE> {
    match self {
      Self::DomainSpecific(t) => {
        ParseError::<'pool, N, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>::domain_err(mapper(t))
      }
      Self::OutOfBounds(e) => {
        ParseError::<'pool, N, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>::OutOfBounds(e)
      }
      Self::UnderlyingProviderReadError(re) => {
        ParseError::<'pool, N, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>::UnderlyingProviderReadError(re)
      }
      Self::UnderlyingProviderStatError(se) => {
        ParseError::<'pool, N, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>::UnderlyingProviderStatError(se)
      }
    }
  }
}

impl<
    'pool,
    T: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
    Re: ProviderError,
    Se: ProviderError,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > From<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>
  for ParseError<'pool, T, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn from(value: ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::OutOfBounds(value)
  }
}

impl<
    'pool,
    T: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
    Re: ProviderError,
    Se: ProviderError,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > From<DomainError<T>> for ParseError<'pool, T, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn from(value: DomainError<T>) -> Self { Self::DomainSpecific(value.0) }
}

impl<
    'pool,
    T: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
    Re: ProviderError,
    Se: ProviderError,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > From<ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>
  for ParseError<'pool, T, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn from(value: ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    match value {
      ParsePrimitiveError::UnderlyingProviderReadError(re) => Self::UnderlyingProviderReadError(re),
      ParsePrimitiveError::UnderlyingProviderStatError(se) => Self::UnderlyingProviderStatError(se),
      ParsePrimitiveError::OutOfBounds(oob) => Self::OutOfBounds(oob),
    }
  }
}

impl<
    'pool,
    T: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
    Re: ProviderError,
    Se: ProviderError,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > Error<DIAGNOSTIC_NODE_NAME_SIZE> for ParseError<'pool, T, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      Self::OutOfBounds(oob) => oob.with_report(callback),
      Self::DomainSpecific(dse) => dse.with_report(callback),
      Self::UnderlyingProviderReadError(UnderlyingProviderReadError(ure, location)) => {
        ure.with_report(Some(*location), callback)
      }
      Self::UnderlyingProviderStatError(UnderlyingProviderStatError(ure)) => {
        ure.with_report(None, callback)
      }
    }
  }
}
