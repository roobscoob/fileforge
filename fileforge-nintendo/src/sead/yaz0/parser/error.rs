use fileforge::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::FileforgeError,
  stream::error::{
    stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_read::StreamReadError, stream_restore::StreamRestoreError, stream_skip::StreamSkipError,
    user_mutate::UserMutateError, user_overwrite::UserOverwriteError, user_read::UserReadError, user_restore::UserRestoreError, user_skip::UserSkipError,
  },
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
  ReadError(Component, SURE),
}

#[derive(Debug)]
pub enum Yaz0ParserSkipError<SURE: UserReadError, SUSE: UserSkipError> {
  ReadFailed(Component, StreamReadError<SURE>),
  ReadError(Component, SURE),
  SkipFailed(Component, StreamSkipError<SUSE>),
}

#[derive(Debug)]
pub enum Yaz0ParserMutateError<SURE: UserReadError, SUREE: UserRestoreError, SUSE: UserSkipError, SUOE: UserOverwriteError, SUME: UserMutateError> {
  ReadFailed(Yaz0ParserError<SURE>),
  RestoreFailed(StreamRestoreError<SUREE>),
  SkipHeaderFailed(StreamSkipError<SUSE>),
  SkipOperationFailed(StreamSkipError<SUSE>),
  MutateHeaderFailed(StreamMutateError<SUME>),
  OverwriteLiteralFailed(StreamOverwriteError<SUOE>),
  OverwriteShortReadbackFailed(StreamOverwriteError<SUOE>),
  ShrinkageBlocked,
  RemoveHeaderFailed(StreamOverwriteError<SUOE>),
  RemoveReadbackFailed(StreamOverwriteError<SUOE>),
  CreateHeaderFailed(StreamOverwriteError<SUOE>),
}

impl<SURE: UserReadError> UserReadError for Yaz0ParserError<SURE> {}
impl<SURE: UserReadError, SUSE: UserSkipError> UserSkipError for Yaz0ParserSkipError<SURE, SUSE> {}
impl<SURE: UserReadError, SUREE: UserRestoreError, SUSE: UserSkipError, SUOE: UserOverwriteError, SUME: UserMutateError> UserMutateError for Yaz0ParserMutateError<SURE, SUREE, SUSE, SUOE, SUME> {}
impl<SURE: UserReadError, SUREE: UserRestoreError, SUSE: UserSkipError, SUOE: UserOverwriteError, SUME: UserMutateError> UserOverwriteError for Yaz0ParserMutateError<SURE, SUREE, SUSE, SUOE, SUME> {}

impl<SURE: UserReadError> FileforgeError for Yaz0ParserError<SURE> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(fileforge::error::report::Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}

impl<SURE: UserReadError, SUSE: UserSkipError> FileforgeError for Yaz0ParserSkipError<SURE, SUSE> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(fileforge::error::report::Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}

impl<SURE: UserReadError, SUREE: UserRestoreError, SUSE: UserSkipError, SUOE: UserOverwriteError, SUME: UserMutateError> FileforgeError for Yaz0ParserMutateError<SURE, SUREE, SUSE, SUOE, SUME> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(fileforge::error::report::Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}
