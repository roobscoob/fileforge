use fileforge::{
  binary_reader::error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
  error::ext::annotations::annotated::Annotated,
  stream::error::user_read::UserReadError,
};
use fileforge_macros::FileforgeError;

use crate::byte_order_mark::error::invalid::ByteOrderMarkInvalid;

pub mod invalid;

#[derive(FileforgeError)]
pub enum ByteOrderMarkError<'pool, U: UserReadError> {
  Failed(#[from] Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  Invalid(#[from] ByteOrderMarkInvalid<'pool>),
}
