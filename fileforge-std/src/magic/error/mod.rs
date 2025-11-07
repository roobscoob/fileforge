use fileforge::{
  binary_reader::error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
  diagnostic::value::DiagnosticValue,
  error::ext::annotations::annotated::Annotated,
  stream::error::user_read::UserReadError,
};
use fileforge_macros::FileforgeError;

use crate::magic::Magic;
use fileforge::error::render::builtin::number::formatted_unsigned::FormattedExt;

pub mod invalid;

// #[derive(FileforgeError)]
// pub enum MagicError<'pool, const MAGIC_SIZE: usize, U: UserReadError> {
//   Failed(#[from] Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),

//   #[error("Invalid Magic")]
//   Invalid {
//     actual: DiagnosticValue<'pool, Magic<MAGIC_SIZE>>,
//     expected: Magic<MAGIC_SIZE>,
//   },
// }

#[derive(FileforgeError)]
pub enum MagicError<'pool, const MAGIC_SIZE: usize, U: UserReadError> {
  Failed(#[from] Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),

  #[report(&"Invalid Magic")]
  #[flag("I can access fields in here {bound}", bound = some_num.format().base(16))]
  Invalid {
    some_num: u32,
    actual: DiagnosticValue<'pool, Magic<MAGIC_SIZE>>,
    expected: Magic<MAGIC_SIZE>,
  },
}
