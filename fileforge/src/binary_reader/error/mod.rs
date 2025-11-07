use fileforge_macros::FileforgeError;

use crate::{
  binary_reader::error::{common::Read, exhausted::ReaderExhaustedError, seek_out_of_bounds::SeekOutOfBounds},
  stream::error::{user_mutate::UserMutateError, user_partition::UserPartitionError, user_read::UserReadError, user_rewind::UserRewindError, user_skip::UserSkipError},
};

pub mod common;
pub mod exhausted;
pub mod primitive_name_annotation;
pub mod seek_out_of_bounds;

#[derive(FileforgeError)]
pub enum StaticSubforkError<'pool, User: UserPartitionError> {
  Stream(User),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

#[derive(FileforgeError)]
pub enum DynamicSubforkError<'pool, User: UserPartitionError> {
  Stream(User),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

#[derive(FileforgeError)]
pub enum RewindError<'pool, User: UserRewindError> {
  User(User),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

#[derive(FileforgeError)]
pub enum SkipError<'pool, User: UserSkipError> {
  User(User),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

#[derive(FileforgeError)]
pub enum GetPrimitiveError<'pool, User: UserReadError> {
  User(#[from] User),
  ReaderExhausted(#[from] ReaderExhaustedError<'pool, Read>),
}

#[derive(FileforgeError)]
pub enum SetPrimitiveError<'pool, User: UserMutateError> {
  User(#[from] User),
  ReaderExhausted(#[from] ReaderExhaustedError<'pool, Read>),
}
