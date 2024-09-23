use super::parse::ParseError;

pub type Result<'pool, S, T, Re, const DIAGNOSTIC_NODE_NAME_SIZE: usize> = core::result::Result<S, ParseError<'pool, T, Re, DIAGNOSTIC_NODE_NAME_SIZE>>;