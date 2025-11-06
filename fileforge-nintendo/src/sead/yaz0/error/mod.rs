pub mod overwrite;

use fileforge::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::FileforgeError,
  stream::error::{stream_read::StreamReadError, user_read::UserReadError, user_skip::UserSkipError},
};

use crate::sead::yaz0::{parser::error::Yaz0ParserError, state::malformed_stream::MalformedStream};

#[derive(Debug)]
pub enum Yaz0Error<SURE: UserReadError> {
  MalformedStream(MalformedStream),
  ParseError(StreamReadError<Yaz0ParserError<SURE>>),
}

impl<SURE: UserReadError> UserReadError for Yaz0Error<SURE> {}
impl<SURE: UserReadError> UserSkipError for Yaz0Error<SURE> {}

impl<SURE: UserReadError> FileforgeError for Yaz0Error<SURE> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(fileforge::error::report::Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    match self {
      Self::MalformedStream(_) => panic!("MS"),
      Self::ParseError(StreamReadError::StreamExhausted(se)) => panic!("SE: {se:?}"),
      Self::ParseError(StreamReadError::User(u)) => u.render_into_report(provider, callback),
    }
  }
}
