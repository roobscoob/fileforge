pub mod bool;
pub mod float32;
pub mod integer32;
pub mod null;
pub mod string;
pub mod string_table;
pub mod unsigned_integer32;

use core::future::Future;

use enum_as_inner::EnumAsInner;
use fileforge::{binary_reader::BinaryReader, stream::ReadableStream, ResultIgnoreExt};
use strum::EnumDiscriminants;

use crate::byml::node::{
  bool::BymlBoolNode,
  float32::BymlFloat32Node,
  integer32::BymlInteger32Node,
  null::BymlNullNode,
  string::BymlStringNode,
  string_table::{BymlStringTableNode, BymlStringTableNodeConstructableError},
  unsigned_integer32::BymlUnsignedInteger32Node,
};

#[derive(EnumAsInner, EnumDiscriminants)]
pub enum BymlNode<'pool, S: ReadableStream<Type = u8>> {
  String(BymlStringNode),
  BinaryData(()),
  BinaryDataWithParameter(()),
  Array(()),
  Dictionary(()),
  StringTable(BymlStringTableNode<'pool, S>),
  BinaryDataTable(()),
  Bool(BymlBoolNode),
  Integer32(BymlInteger32Node),
  Float32(BymlFloat32Node),
  UnsignedInteger32(BymlUnsignedInteger32Node),
  Integer64(()),
  UnsignedInteger64(()),
  Float64(()),
  Null(BymlNullNode),
}

impl BymlNodeDiscriminants {
  pub fn resolve(id: u8) -> Option<BymlNodeDiscriminants> {
    Some(match id {
      0xA0 => BymlNodeDiscriminants::String,
      0xA1 => BymlNodeDiscriminants::BinaryData,
      0xA2 => BymlNodeDiscriminants::BinaryDataWithParameter,
      0xC0 => BymlNodeDiscriminants::Array,
      0xC1 => BymlNodeDiscriminants::Dictionary,
      0xC2 => BymlNodeDiscriminants::StringTable,
      0xC3 => BymlNodeDiscriminants::BinaryDataTable,
      0xD0 => BymlNodeDiscriminants::Bool,
      0xD1 => BymlNodeDiscriminants::Integer32,
      0xD2 => BymlNodeDiscriminants::Float32,
      0xD3 => BymlNodeDiscriminants::UnsignedInteger32,
      0xD4 => BymlNodeDiscriminants::Integer64,
      0xD5 => BymlNodeDiscriminants::UnsignedInteger64,
      0xD6 => BymlNodeDiscriminants::Float64,
      0xFF => BymlNodeDiscriminants::Null,
      _ => return None,
    })
  }

  pub fn min_version(&self) -> u16 {
    match self {
      Self::BinaryDataTable => todo!(),

      Self::String => 1,
      Self::Array => 1,
      Self::Dictionary => 1,
      Self::StringTable => 1,
      Self::Bool => 1,
      Self::Integer32 => 1,
      Self::Float32 => 1,
      Self::Null => 1,

      Self::UnsignedInteger32 => 2,

      Self::Integer64 => 3,
      Self::UnsignedInteger64 => 3,
      Self::Float64 => 3,

      Self::BinaryData => 4,

      Self::BinaryDataWithParameter => 5,
    }
  }
}

pub trait BymlConstructable<'pool, S: ReadableStream<Type = u8>, E>: Sized {
  type Error;

  fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(value: u32, get_reader: F) -> impl Future<Output = Result<Result<Self, Self::Error>, E>>;
}

pub trait BymlAnyConstructable<'pool, S: ReadableStream<Type = u8>, E>: Sized {
  type Error;

  fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(discriminant: BymlNodeDiscriminants, value: u32, get_reader: F) -> impl Future<Output = Result<Self, Self::Error>>;
}

pub enum BymlConstructionError<'pool, E, S: ReadableStream<Type = u8>> {
  ReaderAcquire(E),

  StringTable(BymlStringTableNodeConstructableError<'pool, S>),
}

impl<'pool, S: ReadableStream<Type = u8>, E> BymlAnyConstructable<'pool, S, E> for BymlNode<'pool, S> {
  type Error = BymlConstructionError<'pool, E, S>;

  async fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(discriminant: BymlNodeDiscriminants, value: u32, get_reader: F) -> Result<Self, Self::Error> {
    Ok(match discriminant {
      // Trivially Constructable
      BymlNodeDiscriminants::String => BymlNode::String(BymlStringNode::construct(value, get_reader).await.map_err(BymlConstructionError::ReaderAcquire)?.ignore()),
      BymlNodeDiscriminants::Bool => BymlNode::Bool(BymlBoolNode::construct(value, get_reader).await.map_err(BymlConstructionError::ReaderAcquire)?.ignore()),
      BymlNodeDiscriminants::Integer32 => BymlNode::Integer32(BymlInteger32Node::construct(value, get_reader).await.map_err(BymlConstructionError::ReaderAcquire)?.ignore()),
      BymlNodeDiscriminants::Float32 => BymlNode::Float32(BymlFloat32Node::construct(value, get_reader).await.map_err(BymlConstructionError::ReaderAcquire)?.ignore()),
      BymlNodeDiscriminants::UnsignedInteger32 => BymlNode::UnsignedInteger32(BymlUnsignedInteger32Node::construct(value, get_reader).await.map_err(BymlConstructionError::ReaderAcquire)?.ignore()),
      BymlNodeDiscriminants::Null => BymlNode::Null(BymlNullNode::construct(value, get_reader).await.map_err(BymlConstructionError::ReaderAcquire)?.ignore()),

      // Non-Trivial
      BymlNodeDiscriminants::BinaryData => BymlNode::BinaryData(todo!()),
      BymlNodeDiscriminants::BinaryDataWithParameter => BymlNode::BinaryDataWithParameter(todo!()),
      BymlNodeDiscriminants::Array => BymlNode::Array(todo!()),
      BymlNodeDiscriminants::Dictionary => BymlNode::Dictionary(todo!()),
      BymlNodeDiscriminants::StringTable => BymlNode::StringTable(
        BymlStringTableNode::construct(value, get_reader)
          .await
          .map_err(BymlConstructionError::ReaderAcquire)?
          .map_err(BymlConstructionError::StringTable)?,
      ),
      BymlNodeDiscriminants::BinaryDataTable => BymlNode::BinaryDataTable(todo!()),
      BymlNodeDiscriminants::Integer64 => BymlNode::Integer64(todo!()),
      BymlNodeDiscriminants::UnsignedInteger64 => BymlNode::UnsignedInteger64(todo!()),
      BymlNodeDiscriminants::Float64 => BymlNode::Float64(todo!()),
    })
  }
}
