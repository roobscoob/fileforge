use crate::{diagnostic::node::reference::DiagnosticReference, error::{render::{buffer::{cell::{tag::{context::RenderMode, builtin::report::REPORT_FLAG_LINE_TEXT, CellTag}, RenderBufferCell}, RenderBuffer}, position::RenderPosition, builtin::text::Text, r#trait::renderable::Renderable}, report::{Report, kind::ReportKind, note::ReportNote}, Error}, provider::error::{read_error::ReadError, ProviderError}};

use self::{domain::DomainError, out_of_bounds::ReadOutOfBoundsError, underlying_provider_read::UnderlyingProviderReadError};

use super::r#trait::primitive::Primitive;

pub mod out_of_bounds;
pub mod domain;
pub mod result;
pub mod underlying_provider_read;

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

pub struct ExpectationFailedError<'pool, ErrorFn: for <'primitive, 'closure> Fn(&'primitive P, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), P: Primitive<PRIMITIVE_SIZE>, const PRIMITIVE_SIZE: usize, const DIAGNOSTIC_NODE_NAME_SIZE: usize>(pub P, pub DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>, pub ErrorFn);

pub enum ExpectPrimitiveError<'pool, Re: ProviderError, ErrorFn: for <'primitive, 'closure> Fn(&'primitive P, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), P: Primitive<PRIMITIVE_SIZE>, const PRIMITIVE_SIZE: usize, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  OutOfBounds(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderReadError(UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
  ExpectationFailed(ExpectationFailedError<'pool, ErrorFn, P,  PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>),
}

pub trait ExpectPrimitiveErrorResultExtension<'pool, Success, Re: ProviderError, ErrorFn: for <'primitive, 'closure> Fn(&'primitive P, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), P: Primitive<PRIMITIVE_SIZE>, const PRIMITIVE_SIZE: usize, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  fn map_expectation_failed<Ne, Cb: FnOnce(ExpectationFailedError<'pool, ErrorFn, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>) -> Ne>(self, cb: Cb) -> Result<Result<Success, ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne>;
}

impl<'pool, Success, Re: ProviderError, ErrorFn: for <'primitive, 'closure> Fn(&'primitive P, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), P: Primitive<PRIMITIVE_SIZE>, const PRIMITIVE_SIZE: usize, const DIAGNOSTIC_NODE_NAME_SIZE: usize> ExpectPrimitiveErrorResultExtension<'pool, Success, Re, ErrorFn, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE> for Result<Success, ExpectPrimitiveError<'pool, Re, ErrorFn, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>> {
  fn map_expectation_failed<Ne, Cb: FnOnce(ExpectationFailedError<'pool, ErrorFn, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>) -> Ne>(self, cb: Cb) -> Result<Result<Success, ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne> {
    match self {
      Ok(v) => Ok(Ok(v)),
      Err(ExpectPrimitiveError::ExpectationFailed(ef)) => Err(cb(ef)),
      Err(ExpectPrimitiveError::OutOfBounds(oobe)) => Ok(Err(ParsePrimitiveError::OutOfBounds(oobe))),
      Err(ExpectPrimitiveError::UnderlyingProviderReadError(upre)) => Ok(Err(ParsePrimitiveError::UnderlyingProviderReadError(upre)))
    }
  }
}

impl<'pool, Re: ProviderError, ErrorFn: for <'primitive, 'closure> Fn(&'primitive P, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), P: Primitive<PRIMITIVE_SIZE>, const PRIMITIVE_SIZE: usize, const DIAGNOSTIC_NODE_NAME_SIZE: usize> ExpectPrimitiveError<'pool, Re, ErrorFn, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE> {
  pub fn from_read_error(value: ReadError<Re>, location: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::UnderlyingProviderReadError(UnderlyingProviderReadError(value.0, location))
  }
}

impl<'pool, Re: ProviderError, ErrorFn: for <'primitive, 'closure> Fn(&'primitive P, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), P: Primitive<PRIMITIVE_SIZE>, const PRIMITIVE_SIZE: usize, const DIAGNOSTIC_NODE_NAME_SIZE: usize> From<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>> for ExpectPrimitiveError<'pool, Re, ErrorFn, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn from(value: ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::OutOfBounds(value)
  }
}

impl<'pool, Re: ProviderError, ErrorFn: for <'primitive, 'closure> Fn(&'primitive P, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), P: Primitive<PRIMITIVE_SIZE>, const PRIMITIVE_SIZE: usize, const DIAGNOSTIC_NODE_NAME_SIZE: usize> From<ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>> for ExpectPrimitiveError<'pool, Re, ErrorFn, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn from(value: ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    match value {
      ParsePrimitiveError::OutOfBounds(oobe) => oobe.into(),
      ParsePrimitiveError::UnderlyingProviderReadError(read_error) => Self::UnderlyingProviderReadError(read_error)
    }
  }
}

impl<'pool, Re: ProviderError, ErrorFn: for <'primitive, 'closure> Fn(&'primitive P, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), P: Primitive<PRIMITIVE_SIZE>, const PRIMITIVE_SIZE: usize, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for ExpectPrimitiveError<'pool, Re, ErrorFn, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, mut callback: Cb) {
    match self {
      Self::OutOfBounds(oob) => oob.with_report(callback),
      Self::UnderlyingProviderReadError(UnderlyingProviderReadError(ure, location)) => ure.with_report_given_location(*location, callback),
      Self::ExpectationFailed(ExpectationFailedError(primitive, location, builder)) => {
        if !location.family_exists() {
          builder(primitive, &mut |text, _, _| {
            let line = Text::new()
              .push("The diagnostic pool was too small to be able to load the diagnostics for this error. You are seeing a minified version with what available data exists.", &REPORT_FLAG_LINE_TEXT);
  
            callback(
              Report::new::<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(ReportKind::Error, "Expectation Unmet")
                .with_flag_line(&line).unwrap()
                .with_info_line(&text).unwrap()
            )
          });
        }
        
        builder(primitive, &mut |text, tag, renderable| {
          callback(
            Report::new::<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(ReportKind::Error, "Read Out Of Bounds")
              .with_note(|| {
                if let Some(renderable) = renderable {
                  ReportNote::new(&text).with_tag(tag).with_location(*location, renderable)
                } else {
                  ReportNote::new(&text).with_tag(tag).with_unvalued_location(*location)
                }.unwrap()
              }).unwrap()
          )
        })
      }
    }
  }
}
