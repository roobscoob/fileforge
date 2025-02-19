use fileforge_lib::{error::FileforgeError, reader::readable::error::user::UserReadableError};

pub enum Yaz0ConstructionError {
    
}

impl<'pool, const NODE_NAME_SIZE: usize> UserReadableError<'pool, NODE_NAME_SIZE> for Yaz0ConstructionError {}

impl<'pool, const NODE_NAME_SIZE: usize> FileforgeError<'pool, NODE_NAME_SIZE> for Yaz0ConstructionError {
    fn render_into_report(&self, callback: impl for<'a, 'b> FnMut(fileforge_lib::error::report::Report<'a, 'b, 'pool, NODE_NAME_SIZE>) -> ()) {
        unimplemented!()
    }
}
