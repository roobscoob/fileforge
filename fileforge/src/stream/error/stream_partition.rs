use fileforge_macros::story;
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use super::{stream_exhausted::StreamExhaustedError, user_partition::UserPartitionError};

#[story("stream failed", StreamPartitionError::User(StoryUserError))]
#[story("stream exhausted", StreamPartitionError::<StoryUserError>::StreamExhausted(StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 }))]
pub enum StreamPartitionError<UserPartition: UserPartitionError> {
  User(UserPartition),
  StreamExhausted(StreamExhaustedError),
}

impl<U: UserPartitionError> FileforgeError for StreamPartitionError<U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, callback),
      Self::StreamExhausted(error) => error.render_into_report(provider, callback),
    }
  }
}
