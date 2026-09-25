use fileforge_macros::story;
use crate::{
  binary_reader::error::{common::Read, exhausted::ReaderExhaustedError, seek_out_of_bounds::SeekOutOfBounds},
  diagnostic::pool::DiagnosticPoolProvider,
  error::{report::Report, FileforgeError},
  stream::error::{user_mutate::UserMutateError, user_partition::UserPartitionError, user_read::UserReadError, user_rewind::UserRewindError, user_skip::UserSkipError},
};

pub mod common;
pub mod exhausted;
pub mod primitive_name_annotation;
pub mod seek_out_of_bounds;

#[story("stream failed", StaticSubforkError::<StoryUserError>::Stream(StoryUserError))]
#[story("out of bounds", StaticSubforkError::<StoryUserError>::OutOfBounds(SeekOutOfBounds {
  seek_offset: common::SeekOffset::InBounds(12),
  provider_size: dv!("save.bin" / "header.file_size" @ 0..4 [8]),
  container_dr: dr!("save.bin" @ 0..8),
}))]
pub enum StaticSubforkError<'pool, User: UserPartitionError> {
  Stream(User),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

impl<'pool, User: UserPartitionError> FileforgeError for StaticSubforkError<'pool, User> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Stream(inner) => inner.render_into_report(provider, callback),
      Self::OutOfBounds(inner) => inner.render_into_report(provider, callback),
    }
  }
}

#[story("stream failed", DynamicSubforkError::<StoryUserError>::Stream(StoryUserError))]
#[story("out of bounds", DynamicSubforkError::<StoryUserError>::OutOfBounds(SeekOutOfBounds {
  seek_offset: common::SeekOffset::InBounds(12),
  provider_size: dv!("save.bin" / "header.file_size" @ 0..4 [8]),
  container_dr: dr!("save.bin" @ 0..8),
}))]
pub enum DynamicSubforkError<'pool, User: UserPartitionError> {
  Stream(User),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

impl<'pool, User: UserPartitionError> FileforgeError for DynamicSubforkError<'pool, User> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Stream(inner) => inner.render_into_report(provider, callback),
      Self::OutOfBounds(inner) => inner.render_into_report(provider, callback),
    }
  }
}

#[story("stream failed", RewindError::<StoryUserError>::User(StoryUserError))]
#[story("out of bounds", RewindError::<StoryUserError>::OutOfBounds(SeekOutOfBounds {
  seek_offset: common::SeekOffset::Underflow { base_offset: 2, subtract: 6 },
  provider_size: dv!("save.bin" / "header.file_size" @ 0..4 [8]),
  container_dr: dr!("save.bin" @ 0..8),
}))]
pub enum RewindError<'pool, User: UserRewindError> {
  User(User),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

impl<'pool, User: UserRewindError> FileforgeError for RewindError<'pool, User> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(inner) => inner.render_into_report(provider, callback),
      Self::OutOfBounds(inner) => inner.render_into_report(provider, callback),
    }
  }
}

#[story("stream failed", SkipError::<StoryUserError>::User(StoryUserError))]
#[story("out of bounds", SkipError::<StoryUserError>::OutOfBounds(SeekOutOfBounds {
  seek_offset: common::SeekOffset::InBounds(12),
  provider_size: dv!("save.bin" / "header.file_size" @ 0..4 [8]),
  container_dr: dr!("save.bin" @ 0..8),
}))]
pub enum SkipError<'pool, User: UserSkipError> {
  User(User),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

impl<'pool, User: UserSkipError> FileforgeError for SkipError<'pool, User> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(inner) => inner.render_into_report(provider, callback),
      Self::OutOfBounds(inner) => inner.render_into_report(provider, callback),
    }
  }
}

#[story("stream failed", GetPrimitiveError::<StoryUserError>::User(StoryUserError))]
#[story("reader exhausted", GetPrimitiveError::<StoryUserError>::ReaderExhausted(crate::binary_reader::error::exhausted::ReaderExhaustedError {
  container: dr!("save.bin" @ 0..6),
  length: DiagnosticValue(4, None),
  offset: 4,
  stream_length: DiagnosticValue(6, None),
  t: crate::binary_reader::error::common::Read,
}))]
pub enum GetPrimitiveError<'pool, User: UserReadError> {
  User(User),
  ReaderExhausted(ReaderExhaustedError<'pool, Read>),
}

impl<'pool, User: UserReadError> FileforgeError for GetPrimitiveError<'pool, User> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(inner) => inner.render_into_report(provider, callback),
      Self::ReaderExhausted(inner) => inner.render_into_report(provider, callback),
    }
  }
}

impl<'pool, User: UserReadError> From<User> for GetPrimitiveError<'pool, User> {
  fn from(value: User) -> Self {
    Self::User(value)
  }
}

impl<'pool, User: UserReadError> From<ReaderExhaustedError<'pool, Read>> for GetPrimitiveError<'pool, User> {
  fn from(value: ReaderExhaustedError<'pool, Read>) -> Self {
    Self::ReaderExhausted(value)
  }
}

#[story("stream failed", SetPrimitiveError::<StoryUserError>::User(StoryUserError))]
#[story("reader exhausted", SetPrimitiveError::<StoryUserError>::ReaderExhausted(crate::binary_reader::error::exhausted::ReaderExhaustedError {
  container: dr!("save.bin" @ 0..6),
  length: DiagnosticValue(4, None),
  offset: 4,
  stream_length: DiagnosticValue(6, None),
  t: crate::binary_reader::error::common::Read,
}))]
pub enum SetPrimitiveError<'pool, User: UserMutateError> {
  User(User),
  ReaderExhausted(ReaderExhaustedError<'pool, Read>),
}

impl<'pool, User: UserMutateError> FileforgeError for SetPrimitiveError<'pool, User> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(inner) => inner.render_into_report(provider, callback),
      Self::ReaderExhausted(inner) => inner.render_into_report(provider, callback),
    }
  }
}

impl<'pool, User: UserMutateError> From<User> for SetPrimitiveError<'pool, User> {
  fn from(value: User) -> Self {
    Self::User(value)
  }
}

impl<'pool, User: UserMutateError> From<ReaderExhaustedError<'pool, Read>> for SetPrimitiveError<'pool, User> {
  fn from(value: ReaderExhaustedError<'pool, Read>) -> Self {
    Self::ReaderExhausted(value)
  }
}
