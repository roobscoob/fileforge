use crate::{error::Error, provider::r#trait::Provider, reader::{error::ParseError, Reader}};

pub trait DynamicSizeReadable<
  'pool_lifetime,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
>: Sized {
  type Argument;
  type Error: Error<DIAGNOSTIC_NODE_NAME_SIZE>;

  fn read<'rl, RP: Provider>(reader: &mut Reader<'pool_lifetime, 'rl, DIAGNOSTIC_NODE_NAME_SIZE, RP>, argument: Self::Argument) -> Result<Self, ParseError<'pool_lifetime, Self::Error, RP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>>;
  fn get_size() -> Option<u64>;
}

pub trait FixedSizeReadable<
  'pool_lifetime,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  const SIZE: usize
>: Sized {
  type Argument;
  type Error: Error<DIAGNOSTIC_NODE_NAME_SIZE>;

  fn read<'rl, RP: Provider>(reader: &mut Reader<'pool_lifetime, 'rl, DIAGNOSTIC_NODE_NAME_SIZE, RP>, argument: Self::Argument) -> Result<Self, ParseError<'pool_lifetime, Self::Error, RP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>>;
}