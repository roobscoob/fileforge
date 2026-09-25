use fileforge_macros::story;
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use crate::binary_reader::error::common::LOW_LEVEL_ERROR;
use crate::error::render::{buffer::cell::tag::builtin::report::REPORT_ERROR_TEXT, builtin::text::r#const::ConstText};
use super::user_restore::UserRestoreError;

#[story("stream failed", StreamRestoreError::User(StoryUserError))]
#[story("restoring forwards", StreamRestoreError::<StoryUserError>::CannotRestoreForwards)]
#[derive(Debug)]
pub enum StreamRestoreError<UserRestore: UserRestoreError> {
  User(UserRestore),
  CannotRestoreForwards,
}

impl<UserRestore: UserRestoreError> From<UserRestore> for StreamRestoreError<UserRestore> {
  fn from(value: UserRestore) -> Self {
    Self::User(value)
  }
}

const RESTORE_FORWARDS: ConstText = ConstText::new("A snapshot can only move a stream backwards, but this one is ahead of it.", &REPORT_ERROR_TEXT);

impl<U: UserRestoreError> FileforgeError for StreamRestoreError<U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, callback),
      Self::CannotRestoreForwards => Report::new::<Self>(provider, &"Can't restore forwards")
        .with_info_line(&RESTORE_FORWARDS)
        .with_flag_line(LOW_LEVEL_ERROR)
        .apply(callback),
    }
  }
}
