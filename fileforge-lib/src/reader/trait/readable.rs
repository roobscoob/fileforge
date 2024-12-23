use crate::{
  error::Error,
  provider::r#trait::Provider,
  reader::{error::parse::ParseError, Reader},
};

pub trait DynamicSizeReadable<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize>: Sized {
  type Argument;
  type Error: Error<DIAGNOSTIC_NODE_NAME_SIZE>;

  fn read<RP: Provider>(
    reader: &mut Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>,
    argument: Self::Argument,
  ) -> Result<
    Self,
    ParseError<'pool, Self::Error, RP::ReadError, RP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  >;
  fn get_size<RP: Provider>(
    reader: &Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>,
  ) -> Result<
    Option<u64>,
    ParseError<'pool, Self::Error, RP::ReadError, RP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  >;
}

pub trait FixedSizeReadable<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const SIZE: usize>:
  Sized
{
  type Argument;
  type Error: Error<DIAGNOSTIC_NODE_NAME_SIZE>;

  fn read<RP: Provider>(
    reader: &mut Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>,
    argument: Self::Argument,
  ) -> Result<
    Self,
    ParseError<'pool, Self::Error, RP::ReadError, RP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  >;
}
