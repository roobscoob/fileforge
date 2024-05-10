use super::ParseError;

pub type Result<'pool_lifetime, S, T, Re, const DIAGNOSTIC_NODE_NAME_SIZE: usize> = core::result::Result<S, ParseError<'pool_lifetime, T, Re, DIAGNOSTIC_NODE_NAME_SIZE>>;