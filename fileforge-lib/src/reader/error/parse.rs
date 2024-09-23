use crate::{diagnostic::node::reference::DiagnosticReference, error::{report::Report, Error}, provider::error::{read_error::ReadError, ProviderError}};

use super::{domain::DomainError, out_of_bounds::ReadOutOfBoundsError, parse_primitive::ParsePrimitiveError, underlying_provider_read::UnderlyingProviderReadError};

pub enum ParseError<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  OutOfBounds(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderReadError(UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
  DomainSpecific(T),
}

pub trait ParseErrorResultExtension<'pool, Success, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  fn map_domain_specific<Ne, Cb: FnOnce(T) -> Ne>(self, cb: Cb) -> Result<Result<Success, ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne>;
}

impl<'pool, Success, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> ParseErrorResultExtension<'pool, Success, T, Re, DIAGNOSTIC_NODE_NAME_SIZE> for Result<Success, ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE>> {
  fn map_domain_specific<Ne, Cb: FnOnce(T) -> Ne>(self, cb: Cb) -> Result<Result<Success, ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne> {
    match self {
      Ok(v) => Ok(Ok(v)),
      Err(ParseError::DomainSpecific(e)) => Err(cb(e)),
      Err(ParseError::OutOfBounds(oob)) => Ok(Err(ParsePrimitiveError::OutOfBounds(oob))),
      Err(ParseError::UnderlyingProviderReadError(upre)) => Ok(Err(ParsePrimitiveError::UnderlyingProviderReadError(upre)))
    }
  }
}

impl<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  pub fn domain_err(value: T) -> Self {
    ParseError::DomainSpecific(value)
  }

  pub fn from_read_error(value: ReadError<Re>, location: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::UnderlyingProviderReadError(UnderlyingProviderReadError(value.0, location))
  }

  pub fn map_domains<N: Error<DIAGNOSTIC_NODE_NAME_SIZE>, M: FnOnce(T) -> N>(self, mapper: M) -> ParseError<'pool, N, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
    match self {
      Self::DomainSpecific(t) => ParseError::<'pool, N, Re, DIAGNOSTIC_NODE_NAME_SIZE>::domain_err(mapper(t)),
      Self::OutOfBounds(e) => ParseError::<'pool, N, Re, DIAGNOSTIC_NODE_NAME_SIZE>::OutOfBounds(e),
      Self::UnderlyingProviderReadError(re) => ParseError::<'pool, N, Re, DIAGNOSTIC_NODE_NAME_SIZE>::UnderlyingProviderReadError(re)
    }
  }
}

impl<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> From<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>> for ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn from(value: ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::OutOfBounds(value)
  }
}

impl<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> From<DomainError<T>> for ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn from(value: DomainError<T>) -> Self {
    Self::DomainSpecific(value.0)
  }
}

impl<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> From<ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>> for ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn from(value: ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    match value {
      ParsePrimitiveError::UnderlyingProviderReadError(re) => Self::UnderlyingProviderReadError(re),
      ParsePrimitiveError::OutOfBounds(oob) => Self::OutOfBounds(oob),
    }
  }
}

impl<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      Self::OutOfBounds(oob) => oob.with_report(callback),
      Self::DomainSpecific(dse) => dse.with_report(callback),
      Self::UnderlyingProviderReadError(UnderlyingProviderReadError(ure, location)) => ure.with_report_given_location(*location, callback),
    }
  }
}
