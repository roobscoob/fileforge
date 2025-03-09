use fileforge_lib::{error::FileforgeError, reader::error::get_primitive::GetPrimitiveError, stream::error::user_read::UserReadError};

use super::invalid::ByteOrderMarkInvalid;

pub enum ByteOrderMarkError<'pool, U: UserReadError> {
    Failed(GetPrimitiveError<'pool, U>),
    Invalid(ByteOrderMarkInvalid<'pool>),
}

impl<'pool, U: UserReadError> From<GetPrimitiveError<'pool, U>> for ByteOrderMarkError<'pool, U> {
    fn from(value: GetPrimitiveError<'pool, U>) -> Self {
        ByteOrderMarkError::Failed(value)
    }
}

impl<'pool, U: UserReadError> From<ByteOrderMarkInvalid<'pool>> for ByteOrderMarkError<'pool, U> {
    fn from(value: ByteOrderMarkInvalid<'pool>) -> Self {
        ByteOrderMarkError::Invalid(value)
    }
}

impl<'pool, U: UserReadError> FileforgeError for ByteOrderMarkError<'pool, U> {
    fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: fileforge_lib::diagnostic::pool::DiagnosticPoolProvider>(&self, provider: &'pool_ref P, callback: impl for<'tag, 'b, 'p2> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> ()) {
        unimplemented!()
    }
}