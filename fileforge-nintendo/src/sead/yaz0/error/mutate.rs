use fileforge::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::r#const::ConstText},
    report::Report,
    FileforgeError,
  },
  stream::error::{
    stream_overwrite::StreamOverwriteError, stream_read::StreamReadError, stream_restore::StreamRestoreError, user_mutate::UserMutateError,
    user_overwrite::UserOverwriteError, user_read::UserReadError, user_restore::UserRestoreError,
  },
};
use fileforge_macros::story;

use crate::report::render_with_context;

/// A failure changing Yaz0-compressed data in place. Changing bytes reads them, goes back, and
/// overwrites them with the changed ones, so any of those can fail.
#[story("read failed", Yaz0MutateError::<StoryUserError, StoryUserError, StoryUserError>::Read(fileforge::stream::error::stream_read::StreamReadError::User(StoryUserError)))]
#[story("returning failed", Yaz0MutateError::<StoryUserError, StoryUserError, StoryUserError>::Restore(fileforge::stream::error::stream_restore::StreamRestoreError::User(StoryUserError)))]
#[story("overwrite failed", Yaz0MutateError::<StoryUserError, StoryUserError, StoryUserError>::Overwrite(fileforge::stream::error::stream_overwrite::StreamOverwriteError::User(StoryUserError)))]
pub enum Yaz0MutateError<R: UserReadError, Re: UserRestoreError, O: UserOverwriteError> {
  Read(StreamReadError<R>),
  Restore(StreamRestoreError<Re>),
  Overwrite(StreamOverwriteError<O>),
}

impl<R: UserReadError, Re: UserRestoreError, O: UserOverwriteError> UserMutateError for Yaz0MutateError<R, Re, O> {}

const READING: ConstText = ConstText::new("This happened while reading Yaz0-compressed data in order to change it.", &REPORT_INFO_LINE_TEXT);
const RETURNING: ConstText = ConstText::new("This happened while returning to Yaz0-compressed data in order to change it.", &REPORT_INFO_LINE_TEXT);
const OVERWRITING: ConstText = ConstText::new("This happened while writing changed Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);

impl<R: UserReadError, Re: UserRestoreError, O: UserOverwriteError> FileforgeError for Yaz0MutateError<R, Re, O> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Read(error) => render_with_context(error, &READING, provider, callback),
      Self::Restore(error) => render_with_context(error, &RETURNING, provider, callback),
      Self::Overwrite(error) => render_with_context(error, &OVERWRITING, provider, callback),
    }
  }
}
