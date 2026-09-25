use fileforge_macros::story;
pub mod overwrite;

use fileforge::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::FileforgeError,
  stream::error::{stream_read::StreamReadError, user_read::UserReadError, user_skip::UserSkipError},
};

use crate::sead::yaz0::{parser::error::Yaz0ParserError, state::malformed_stream::MalformedStream};
use fileforge::error::render::{buffer::cell::tag::builtin::report::REPORT_ERROR_TEXT, builtin::text::r#const::ConstText};
use fileforge::error::report::Report;
use crate::report::{TRUNCATED};

#[story("compressed data ends early", Yaz0Error::<StoryUserError>::ParseError(StreamReadError::StreamExhausted(fileforge::stream::error::stream_exhausted::StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 })))]
#[story("back-reference before the start of the output", Yaz0Error::<StoryUserError>::MalformedStream(MalformedStream::SeekbackOutOfBounds { seekback_offset: 12, seekback_size: 4 }))]
#[story("compressed data failed to parse", Yaz0Error::<StoryUserError>::ParseError(StreamReadError::User(Yaz0ParserError::ReadFailed(crate::sead::yaz0::parser::error::Component::Literal, StreamReadError::StreamExhausted(fileforge::stream::error::stream_exhausted::StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 })))))]
#[derive(Debug)]
pub enum Yaz0Error<SURE: UserReadError> {
  MalformedStream(MalformedStream),
  ParseError(StreamReadError<Yaz0ParserError<SURE>>),
}

impl<SURE: UserReadError> UserReadError for Yaz0Error<SURE> {}
impl<SURE: UserReadError> UserSkipError for Yaz0Error<SURE> {}

const ENDS_EARLY: ConstText = ConstText::new("The compressed data ends before reaching the decompressed size given in the header.", &REPORT_ERROR_TEXT);

impl<SURE: UserReadError> FileforgeError for Yaz0Error<SURE> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    match self {
      Self::MalformedStream(error) => error.render_into_report(provider, callback),
      Self::ParseError(StreamReadError::User(error)) => error.render_into_report(provider, callback),
      Self::ParseError(StreamReadError::StreamExhausted(_)) => Report::new::<Self>(provider, &"Truncated Yaz0 data")
        .with_info_line(&ENDS_EARLY)
        .with_flag_line(&TRUNCATED)
        .apply(callback),
    }
  }
}
