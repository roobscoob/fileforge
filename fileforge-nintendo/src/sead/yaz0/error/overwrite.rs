use fileforge_lib::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::FileforgeError,
  stream::error::{
    stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_read::StreamReadError, stream_restore::StreamRestoreError, user_mutate::UserMutateError,
    user_overwrite::UserOverwriteError, user_read::UserReadError, user_restore::UserRestoreError, user_skip::UserSkipError,
  },
};

use crate::sead::yaz0::{parser::error::Yaz0ParserError, state::malformed_stream::MalformedStream};

#[derive(Debug)]
pub enum Yaz0OverwriteError<SURE: UserReadError, SUREE: UserRestoreError, SUME: UserMutateError, SUOE: UserOverwriteError> {
  RestoreFailed(StreamRestoreError<SUREE>),
  ReadBlockFailed(StreamReadError<SURE>),
  MutateBlockFailed(StreamMutateError<SUME>),
  OverwriteBlockFailed(StreamOverwriteError<SUOE>),
  MalformedStream(MalformedStream),
}

impl<SURE: UserReadError, SUREE: UserRestoreError, SUME: UserMutateError, SUOE: UserOverwriteError> UserOverwriteError for Yaz0OverwriteError<SURE, SUREE, SUME, SUOE> {}

impl<SURE: UserReadError, SUREE: UserRestoreError, SUME: UserMutateError, SUOE: UserOverwriteError> FileforgeError for Yaz0OverwriteError<SURE, SUREE, SUME, SUOE> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    todo!()
  }
}
