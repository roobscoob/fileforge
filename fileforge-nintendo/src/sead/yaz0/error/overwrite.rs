use fileforge_macros::story;
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
use fileforge::error::render::{buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT}, builtin::text::r#const::ConstText};
use fileforge::error::report::Report;
use crate::report::{render_with_context};

#[story("restore failed", Yaz0OverwriteError::<StoryStream, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::RestoreFailed(StreamRestoreError::User(StoryUserError)))]
#[story("reading a block failed", Yaz0OverwriteError::<StoryStream, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::ReadBlockFailed(StreamReadError::User(StoryUserError)))]
#[story("mutating a block failed", Yaz0OverwriteError::<StoryStream, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::MutateBlockFailed(StreamMutateError::User(StoryUserError)))]
#[story("overwriting a block failed", Yaz0OverwriteError::<StoryStream, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::OverwriteBlockFailed(StreamOverwriteError::User(StoryUserError)))]
#[story("malformed stream", Yaz0OverwriteError::<StoryStream, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::MalformedStream(MalformedStream::SeekbackOutOfBounds { seekback_offset: 12, seekback_size: 4 }))]
#[story("mutating the header failed", Yaz0OverwriteError::<StoryStream, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::MutateHeaderError(fileforge::binary_reader::view::ViewMutateError::Restore(StreamRestoreError::User(StoryUserError))))]
#[story("too much data", Yaz0OverwriteError::<StoryStream, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::TooMuchData)]
#[story("writing a header field failed", Yaz0OverwriteError::<StoryStream, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::MutateHeaderFieldError({
  use fileforge::error::ext::annotations::annotated::AnnotationExt;

  Err::<(), _>(fileforge::binary_reader::error::SetPrimitiveError::User(StoryUserError))
    .annotate(fileforge::binary_reader::error::primitive_name_annotation::PrimitiveName::<fileforge::binary_reader::error::common::Write>::for_type::<u32>())
    .unwrap_err()
}))]
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

const RETURNING_TO_BLOCK: ConstText = ConstText::new("This happened while returning to an earlier block to edit Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const READING_BLOCK: ConstText = ConstText::new("This happened while reading a block of Yaz0-compressed data to edit it.", &REPORT_INFO_LINE_TEXT);
const EDITING_BLOCK: ConstText = ConstText::new("This happened while editing a block of Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const REWRITING_BLOCK: ConstText = ConstText::new("This happened while rewriting a block of Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const UPDATING_SIZE: ConstText = ConstText::new("This happened while updating the decompressed size in the Yaz0 header.", &REPORT_INFO_LINE_TEXT);
const WRITING_SIZE: ConstText = ConstText::new("This happened while writing the decompressed size into the Yaz0 header.", &REPORT_INFO_LINE_TEXT);
const TOO_MUCH_DATA: ConstText = ConstText::new("After this edit, the data would be over 4 GiB, which is more than a Yaz0 header can describe.", &REPORT_ERROR_TEXT);

impl<'pool, S: MutableStream<Type = u8> + RestorableStream, SURE: UserReadError, SUREE: UserRestoreError, SUME: UserMutateError, SUOE: UserOverwriteError> FileforgeError
  for Yaz0OverwriteError<'pool, S, SURE, SUREE, SUME, SUOE>
{
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    match self {
      Self::RestoreFailed(error) => render_with_context(error, &RETURNING_TO_BLOCK, provider, callback),
      Self::ReadBlockFailed(error) => render_with_context(error, &READING_BLOCK, provider, callback),
      Self::MutateBlockFailed(error) => render_with_context(error, &EDITING_BLOCK, provider, callback),
      Self::OverwriteBlockFailed(error) => render_with_context(error, &REWRITING_BLOCK, provider, callback),
      Self::MalformedStream(error) => error.render_into_report(provider, callback),
      Self::MutateHeaderError(error) => render_with_context(error, &UPDATING_SIZE, provider, callback),
      Self::TooMuchData => Report::new::<Self>(provider, &"Decompressed data too large")
        .with_info_line(&TOO_MUCH_DATA)
        .apply(callback),
      Self::MutateHeaderFieldError(write) => render_with_context(write.error(), &WRITING_SIZE, provider, callback),
    }
  }
}
