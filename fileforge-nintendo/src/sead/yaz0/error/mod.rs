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
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}
