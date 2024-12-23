use fileforge_macros::text;

use crate::{
  diagnostic::node::reference::DiagnosticReference,
  error::{
    render::{
      buffer::cell::tag::{builtin::report::REPORT_FLAG_LINE_TEXT, CellTag},
      builtin::text::Text,
      r#trait::renderable::Renderable,
    },
    report::{kind::ReportKind, note::ReportNote, Report},
    Error,
  },
  provider::error::{read_error::ReadError, ProviderError},
  reader::r#trait::primitive::Primitive,
};

use super::{
  out_of_bounds::ReadOutOfBoundsError, parse_primitive::ParsePrimitiveError,
  underlying_provider_read::UnderlyingProviderReadError,
  underlying_provider_stat::UnderlyingProviderStatError,
};

pub struct ExpectationFailedError<
  'pool,
  P: Primitive<PRIMITIVE_SIZE>,
  const PRIMITIVE_SIZE: usize,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
>(
  pub P,
  pub DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  pub  for<'f, 'g> fn(
    &'f P,
    &'g mut (dyn for<'tag, 'text_data, 'renderable> FnMut(
      Text<'text_data, 'tag>,
      &'tag (dyn CellTag + 'tag),
      Option<&'renderable (dyn Renderable<'tag> + 'renderable)>,
    ) + 'g),
  ),
);

pub enum ExpectPrimitiveError<
  'pool,
  Re: ProviderError,
  Se: ProviderError,
  P: Primitive<PRIMITIVE_SIZE>,
  const PRIMITIVE_SIZE: usize,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
> {
  OutOfBounds(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderReadError(UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderStatError(UnderlyingProviderStatError<Se>),
  ExpectationFailed(ExpectationFailedError<'pool, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>),
}

pub trait ExpectPrimitiveErrorResultExtension<
  'pool,
  Success,
  Re: ProviderError,
  Se: ProviderError,
  P: Primitive<PRIMITIVE_SIZE>,
  const PRIMITIVE_SIZE: usize,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
>
{
  fn map_expectation_failed<
    Ne,
    Cb: FnOnce(ExpectationFailedError<'pool, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>) -> Ne,
  >(
    self,
    cb: Cb,
  ) -> Result<Result<Success, ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne>;
}

impl<
    'pool,
    Success,
    Re: ProviderError,
    Se: ProviderError,
    P: Primitive<PRIMITIVE_SIZE>,
    const PRIMITIVE_SIZE: usize,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  >
  ExpectPrimitiveErrorResultExtension<
    'pool,
    Success,
    Re,
    Se,
    P,
    PRIMITIVE_SIZE,
    DIAGNOSTIC_NODE_NAME_SIZE,
  >
  for Result<
    Success,
    ExpectPrimitiveError<'pool, Re, Se, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>,
  >
{
  fn map_expectation_failed<
    Ne,
    Cb: FnOnce(ExpectationFailedError<'pool, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>) -> Ne,
  >(
    self,
    cb: Cb,
  ) -> Result<Result<Success, ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>, Ne>
  {
    match self {
      Ok(v) => Ok(Ok(v)),
      Err(ExpectPrimitiveError::ExpectationFailed(ef)) => Err(cb(ef)),
      Err(ExpectPrimitiveError::OutOfBounds(oobe)) => {
        Ok(Err(ParsePrimitiveError::OutOfBounds(oobe)))
      }
      Err(ExpectPrimitiveError::UnderlyingProviderReadError(upre)) => {
        Ok(Err(ParsePrimitiveError::UnderlyingProviderReadError(upre)))
      }
      Err(ExpectPrimitiveError::UnderlyingProviderStatError(upre)) => {
        Ok(Err(ParsePrimitiveError::UnderlyingProviderStatError(upre)))
      }
    }
  }
}

impl<
    'pool,
    Re: ProviderError,
    Se: ProviderError,
    P: Primitive<PRIMITIVE_SIZE>,
    const PRIMITIVE_SIZE: usize,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > ExpectPrimitiveError<'pool, Re, Se, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>
{
  pub fn from_read_error(
    value: ReadError<Re>,
    location: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Self {
    Self::UnderlyingProviderReadError(UnderlyingProviderReadError(value.0, location))
  }
}

impl<
    'pool,
    Re: ProviderError,
    Se: ProviderError,
    P: Primitive<PRIMITIVE_SIZE>,
    const PRIMITIVE_SIZE: usize,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > From<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>
  for ExpectPrimitiveError<'pool, Re, Se, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn from(value: ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::OutOfBounds(value)
  }
}

impl<
    'pool,
    Re: ProviderError,
    Se: ProviderError,
    P: Primitive<PRIMITIVE_SIZE>,
    const PRIMITIVE_SIZE: usize,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > From<ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>>
  for ExpectPrimitiveError<'pool, Re, Se, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn from(value: ParsePrimitiveError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    match value {
      ParsePrimitiveError::OutOfBounds(oobe) => oobe.into(),
      ParsePrimitiveError::UnderlyingProviderReadError(read_error) => {
        Self::UnderlyingProviderReadError(read_error)
      }
      ParsePrimitiveError::UnderlyingProviderStatError(stat_error) => {
        Self::UnderlyingProviderStatError(stat_error)
      }
    }
  }
}

impl<
    'pool,
    Re: ProviderError,
    Se: ProviderError,
    P: Primitive<PRIMITIVE_SIZE>,
    const PRIMITIVE_SIZE: usize,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for ExpectPrimitiveError<'pool, Re, Se, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>
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
      Self::ExpectationFailed(efe) => efe.with_report(callback),
    }
  }
}

impl<
    'pool,
    P: Primitive<PRIMITIVE_SIZE>,
    const PRIMITIVE_SIZE: usize,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  > Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for ExpectationFailedError<'pool, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, mut callback: Cb) {
    if !self.1.family_exists() {
      self.2(&self.0, &mut |text, _, _| {
        callback(
          Report::new::<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(ReportKind::Error, "Expectation Unmet")
            .with_flag_line(&text!([&REPORT_FLAG_LINE_TEXT] "The diagnostic pool was too small to be able to load the diagnostics for this error. You are seeing a minified version with what available data exists.")).unwrap()
            .with_info_line(&text).unwrap()
        )
      });
    }

    self.2(&self.0, &mut |text, tag, renderable| {
      callback(
        Report::new::<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(
          ReportKind::Error,
          "Read Out Of Bounds",
        )
        .with_note(|| {
          if let Some(renderable) = renderable {
            ReportNote::new(&text)
              .with_tag(tag)
              .with_location(self.1, renderable)
          } else {
            ReportNote::new(&text)
              .with_tag(tag)
              .with_unvalued_location(self.1)
          }
          .unwrap()
        })
        .unwrap(),
      )
    })
  }
}
