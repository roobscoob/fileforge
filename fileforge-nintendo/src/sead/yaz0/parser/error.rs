use fileforge_lib::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::FileforgeError,
  stream::error::{stream_read::StreamReadError, stream_skip::StreamSkipError, user_mutate::UserMutateError, user_read::UserReadError, user_skip::UserSkipError},
};

#[derive(Debug)]
pub enum Component {
  Header,
  Literal,
  SequenceHeader,
  SmallSequenceTail,
  LargeSequenceTail,
}

#[derive(Debug)]
pub enum Yaz0ParserError<SURE: UserReadError> {
  ReadFailed(Component, StreamReadError<SURE>),
}

#[derive(Debug)]
pub enum Yaz0ParserSkipError<SURE: UserReadError, SUSE: UserSkipError> {
  ReadFailed(Component, StreamReadError<SURE>),
  SkipFailed(Component, StreamSkipError<SUSE>),
}

pub enum Yaz0ParserMutateError {}

impl<SURE: UserReadError> UserReadError for Yaz0ParserError<SURE> {}
impl<SURE: UserReadError, SUSE: UserSkipError> UserSkipError for Yaz0ParserSkipError<SURE, SUSE> {}
impl UserMutateError for Yaz0ParserMutateError {}

impl<SURE: UserReadError> FileforgeError for Yaz0ParserError<SURE> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}

impl<SURE: UserReadError, SUSE: UserSkipError> FileforgeError for Yaz0ParserSkipError<SURE, SUSE> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}

impl FileforgeError for Yaz0ParserMutateError {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}
