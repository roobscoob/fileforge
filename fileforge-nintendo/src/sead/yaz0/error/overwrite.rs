use fileforge::{
  binary_reader::{
    error::{primitive_name_annotation::PrimitiveName, SetPrimitiveError},
    view::ViewMutateError,
  },
  diagnostic::pool::DiagnosticPoolProvider,
  error::{ext::annotations::annotated::Annotated, FileforgeError},
  stream::{
    error::{
      stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_read::StreamReadError, stream_restore::StreamRestoreError, user_mutate::UserMutateError,
      user_overwrite::UserOverwriteError, user_read::UserReadError, user_restore::UserRestoreError,
    },
    MutableStream, RestorableStream,
  },
};

use crate::sead::yaz0::{header::Yaz0Header, state::malformed_stream::MalformedStream};

pub enum Yaz0OverwriteError<'pool, S: MutableStream<Type = u8> + RestorableStream, SURE: UserReadError, SUREE: UserRestoreError, SUME: UserMutateError, SUOE: UserOverwriteError> {
  RestoreFailed(StreamRestoreError<SUREE>),
  ReadBlockFailed(StreamReadError<SURE>),
  MutateBlockFailed(StreamMutateError<SUME>),
  OverwriteBlockFailed(StreamOverwriteError<SUOE>),
  MalformedStream(MalformedStream),
  MutateHeaderError(ViewMutateError<'pool, S, Yaz0Header>),
  TooMuchData,
  MutateHeaderFieldError(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Write>, SetPrimitiveError<'pool, <S as MutableStream>::MutateError>>),
}

impl<'pool, S: MutableStream<Type = u8> + RestorableStream, SURE: UserReadError, SUREE: UserRestoreError, SUME: UserMutateError, SUOE: UserOverwriteError> UserOverwriteError
  for Yaz0OverwriteError<'pool, S, SURE, SUREE, SUME, SUOE>
{
}

impl<'pool, S: MutableStream<Type = u8> + RestorableStream, SURE: UserReadError, SUREE: UserRestoreError, SUME: UserMutateError, SUOE: UserOverwriteError> FileforgeError
  for Yaz0OverwriteError<'pool, S, SURE, SUREE, SUME, SUOE>
{
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}
