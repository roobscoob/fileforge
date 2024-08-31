use crate::{provider::r#trait::Provider, reader::{r#trait::readable::DynamicSizeReadable, Reader}};

use super::file::BymlFile;

pub mod array;

pub trait BymlNodeReader<'file, 'pool, 'provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider>: Sized {
  fn from_value(value: u32, file: &BymlFile<'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P>) -> Self;

  fn from_data(data: &mut Reader<'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P>, file: &BymlFile<'pool, 'provider, DIAGNOSTIC_NODE_NAME_SIZE, P>) -> Self;

  fn id() -> u8;
}