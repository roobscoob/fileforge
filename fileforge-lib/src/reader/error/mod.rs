use core::fmt::Debug;

use alloc::vec;

use crate::{diagnostic::node::reference::DiagnosticReference, error::{render::{buffer::{cell::{tag::context::RenderMode, RenderBufferCell}, RenderBuffer}, position::RenderPosition}, report::Report, Error}, provider::error::{read_error::ReadError, ProviderError}};

use self::{domain::DomainError, out_of_bounds::ReadOutOfBoundsError};

pub mod out_of_bounds;
pub mod domain;
pub mod result;

pub enum ParseError<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  OutOfBounds(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderReadError(Re, DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  DomainSpecific(T),
}

impl<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  pub fn domain_err(value: T) -> Self {
    ParseError::DomainSpecific(value)
  }

  pub fn from_read_error(value: ReadError<Re>, location: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::UnderlyingProviderReadError(value.0, location)
  }

  pub fn map_domains<N: Error<DIAGNOSTIC_NODE_NAME_SIZE>, M: FnOnce(T) -> N>(self, mapper: M) -> ParseError<'pool, N, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
    match self {
      Self::DomainSpecific(t) => ParseError::<'pool, N, Re, DIAGNOSTIC_NODE_NAME_SIZE>::domain_err(mapper(t)),
      Self::OutOfBounds(e) => ParseError::<'pool, N, Re, DIAGNOSTIC_NODE_NAME_SIZE>::OutOfBounds(e),
      Self::UnderlyingProviderReadError(re, refr) => ParseError::<'pool, N, Re, DIAGNOSTIC_NODE_NAME_SIZE>::UnderlyingProviderReadError(re, refr)
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
      ParsePrimitiveError::UnderlyingProviderReadError(re, l) => Self::UnderlyingProviderReadError(re, l),
      ParsePrimitiveError::OutOfBounds(oob) => Self::OutOfBounds(oob),
    }
  }
}

impl<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      Self::OutOfBounds(oob) => oob.with_report(callback),
      Self::DomainSpecific(dse) => dse.with_report(callback),
      Self::UnderlyingProviderReadError(ure, location) => ure.with_report_given_location(*location, callback),
    }
  }
}

#[cfg(feature = "alloc")]
impl<'pool, T: Error<DIAGNOSTIC_NODE_NAME_SIZE>, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Debug for ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut result = Ok(());

    f.write_str("\n")?;

    self.with_report(|report| {
      let mut buffer = RenderBuffer::dry();
      let mut canvas = buffer.canvas_at(RenderPosition::zero());
      canvas.write(&report).unwrap();

      let width = buffer.highest_written_column + 1;

      let mut vec = vec![RenderBufferCell::default(); width];
      let mut slice = vec.as_mut_slice();
      let mut i = 0;
      
      loop {
        let mut buffer = RenderBuffer::new(&mut slice, width, i);
        let mut canvas = buffer.canvas_at(RenderPosition::zero());

        canvas.write(&report).unwrap();

        if buffer.is_empty() {
          break;
        }

        result = buffer.flush_into(f, RenderMode::TerminalAnsi);

        if result.is_err() {
          return;
        }

        for cell in slice.iter_mut() {
          cell.clear();
        }

        i += 1;
      }
    });

    result
  }
}

pub enum ParsePrimitiveError<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  OutOfBounds(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnderlyingProviderReadError(Re, DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
}

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  pub fn from_read_error(value: ReadError<Re>, location: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::UnderlyingProviderReadError(value.0, location)
  }
}

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> From<ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>> for ParsePrimitiveError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn from(value: ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::OutOfBounds(value)
  }
}

