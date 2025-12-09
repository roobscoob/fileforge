pub mod bool;
pub mod discriminant;
pub mod float32;
pub mod integer32;
pub mod null;
pub mod string;
pub mod string_table;
pub mod unsigned_integer32;

use core::future::Future;

use enum_as_inner::EnumAsInner;
use fileforge::{binary_reader::BinaryReader, stream::ReadableStream, ResultIgnoreExt};
use fileforge_macros::FileforgeError;
use strum::EnumDiscriminants;

use crate::byml::node::{
  bool::BymlBoolNode,
  discriminant::{BymlNodeDiscriminantVersionConfig, BymlNodeDiscriminantsReadError},
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

pub(super) trait BymlConstructable<'pool, S: ReadableStream<Type = u8>, E>: Sized {
  type Error;

  fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(value: u32, get_reader: F) -> impl Future<Output = Result<Self, Self::Error>>;
}

pub(super) trait BymlDynConstructable<'pool, S: ReadableStream<Type = u8>>: Sized {
  type Error;

  fn construct_dyn(reader: BinaryReader<'pool, S>) -> impl Future<Output = Result<Self, Self::Error>>;
}

async fn construct_as_dyn<'pool, T: BymlDynConstructable<'pool, S>, S: ReadableStream<Type = u8>, E, F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(
  discriminant: BymlNodeDiscriminants,
  value: u32,
  version: BymlNodeDiscriminantVersionConfig,
  get_reader: F,
) -> Result<T, BymlDynConstructableError<'pool, S, E, T::Error>> {
  let mut reader = get_reader(value as u64).await.map_err(BymlDynConstructableError::ReaderAcquire)?;
  let in_place_discriminant = reader.read_with(version).await.map_err(BymlDynConstructableError::ReadType)?;

  (discriminant == in_place_discriminant)
    .then(|| T::construct_dyn(reader))
    .ok_or(BymlDynConstructableError::InvalidDynConstructable(in_place_discriminant))?
    .await
    .map_err(BymlDynConstructableError::Item)
}

enum BymlDynConstructableError<'pool, S: ReadableStream<Type = u8>, E, I> {
  ReaderAcquire(E),
  ReadType(BymlNodeDiscriminantsReadError<'pool, S>),
  InvalidDynConstructable(BymlNodeDiscriminants),
  Item(I),
}

impl<'pool, S: ReadableStream<Type = u8>, E, I> BymlDynConstructableError<'pool, S, E, I> {
  pub fn map_item(self, into: impl FnOnce(I) -> BymlConstructionError<'pool, E, S>) -> BymlConstructionError<'pool, E, S> {
    match self {
      Self::ReaderAcquire(e) => BymlConstructionError::ReaderAcquire(e),
      Self::ReadType(e) => BymlConstructionError::ReadType(e),
      Self::InvalidDynConstructable(e) => BymlConstructionError::InvalidDynConstructable(e),
      Self::Item(i) => into(i),
    }
  }
}

pub trait BymlAnyConstructable<'pool, S: ReadableStream<Type = u8>, E>: Sized {
  type Error;
  type DynError;

  fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(
    discriminant: BymlNodeDiscriminants,
    value: u32,
    version: BymlNodeDiscriminantVersionConfig,
    get_reader: F,
  ) -> impl Future<Output = Result<Self, Self::Error>>;

  fn construct_dyn<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(
    value: u32,
    version: BymlNodeDiscriminantVersionConfig,
    get_reader: F,
  ) -> impl Future<Output = Result<Self, Self::DynError>>;
}

#[derive(FileforgeError)]
pub enum BymlConstructionError<'pool, E, S: ReadableStream<Type = u8>> {
  ReaderAcquire(E),
  ReadType(#[from] BymlNodeDiscriminantsReadError<'pool, S>),

  #[report(&"Invalid Dyn Constructable :(")]
  InvalidDynConstructable(BymlNodeDiscriminants),

  StringTable(#[from] BymlStringTableNodeConstructableError<'pool, S>),
}

impl<'pool, S: ReadableStream<Type = u8>, E> BymlAnyConstructable<'pool, S, E> for BymlNode<'pool, S> {
  type Error = BymlConstructionError<'pool, E, S>;
  type DynError = BymlConstructionError<'pool, E, S>;

  async fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(
    discriminant: BymlNodeDiscriminants,
    value: u32,
    version: BymlNodeDiscriminantVersionConfig,
    get_reader: F,
  ) -> Result<Self, Self::Error> {
    Ok(match discriminant {
      // Trivially Constructable
      BymlNodeDiscriminants::String => BymlNode::String(BymlStringNode::construct(value, get_reader).await.ignore()),
      BymlNodeDiscriminants::Bool => BymlNode::Bool(BymlBoolNode::construct(value, get_reader).await.ignore()),
      BymlNodeDiscriminants::Integer32 => BymlNode::Integer32(BymlInteger32Node::construct(value, get_reader).await.ignore()),
      BymlNodeDiscriminants::Float32 => BymlNode::Float32(BymlFloat32Node::construct(value, get_reader).await.ignore()),
      BymlNodeDiscriminants::UnsignedInteger32 => BymlNode::UnsignedInteger32(BymlUnsignedInteger32Node::construct(value, get_reader).await.ignore()),
      BymlNodeDiscriminants::Null => BymlNode::Null(BymlNullNode::construct(value, get_reader).await.ignore()),

      // Non-Trivial
      BymlNodeDiscriminants::BinaryData => BymlNode::BinaryData(todo!()),
      BymlNodeDiscriminants::BinaryDataWithParameter => BymlNode::BinaryDataWithParameter(todo!()),
      BymlNodeDiscriminants::Array => BymlNode::Array(todo!()),
      BymlNodeDiscriminants::Dictionary => BymlNode::Dictionary(todo!()),
      BymlNodeDiscriminants::StringTable => BymlNode::StringTable(
        construct_as_dyn(discriminant, value, version, get_reader)
          .await
          .map_err(|e| e.map_item(BymlConstructionError::StringTable))?,
      ),
      BymlNodeDiscriminants::BinaryDataTable => BymlNode::BinaryDataTable(todo!()),
      BymlNodeDiscriminants::Integer64 => BymlNode::Integer64(todo!()),
      BymlNodeDiscriminants::UnsignedInteger64 => BymlNode::UnsignedInteger64(todo!()),
      BymlNodeDiscriminants::Float64 => BymlNode::Float64(todo!()),
    })
  }

  async fn construct_dyn<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(value: u32, version: BymlNodeDiscriminantVersionConfig, get_reader: F) -> Result<Self, Self::DynError> {
    let mut reader = get_reader(value as u64).await.map_err(BymlConstructionError::ReaderAcquire)?;

    Ok(match reader.read_with(version).await.map_err(BymlConstructionError::ReadType)? {
      // Trivially Constructable
      BymlNodeDiscriminants::String => return Err(BymlConstructionError::InvalidDynConstructable(BymlNodeDiscriminants::String)),
      BymlNodeDiscriminants::Bool => return Err(BymlConstructionError::InvalidDynConstructable(BymlNodeDiscriminants::Bool)),
      BymlNodeDiscriminants::Integer32 => return Err(BymlConstructionError::InvalidDynConstructable(BymlNodeDiscriminants::Integer32)),
      BymlNodeDiscriminants::Float32 => return Err(BymlConstructionError::InvalidDynConstructable(BymlNodeDiscriminants::Float32)),
      BymlNodeDiscriminants::UnsignedInteger32 => return Err(BymlConstructionError::InvalidDynConstructable(BymlNodeDiscriminants::UnsignedInteger32)),
      BymlNodeDiscriminants::Null => return Err(BymlConstructionError::InvalidDynConstructable(BymlNodeDiscriminants::Null)),

      // Non-Trivial
      BymlNodeDiscriminants::BinaryData => BymlNode::BinaryData(todo!()),
      BymlNodeDiscriminants::BinaryDataWithParameter => BymlNode::BinaryDataWithParameter(todo!()),
      BymlNodeDiscriminants::Array => BymlNode::Array(todo!()),
      BymlNodeDiscriminants::Dictionary => BymlNode::Dictionary(todo!()),
      BymlNodeDiscriminants::StringTable => BymlNode::StringTable(BymlStringTableNode::construct_dyn(reader).await.map_err(BymlConstructionError::StringTable)?),
      BymlNodeDiscriminants::BinaryDataTable => BymlNode::BinaryDataTable(todo!()),
      BymlNodeDiscriminants::Integer64 => BymlNode::Integer64(todo!()),
      BymlNodeDiscriminants::UnsignedInteger64 => BymlNode::UnsignedInteger64(todo!()),
      BymlNodeDiscriminants::Float64 => BymlNode::Float64(todo!()),
    })
  }
}
