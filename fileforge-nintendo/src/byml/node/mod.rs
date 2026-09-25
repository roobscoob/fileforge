use fileforge_macros::story;
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
use fileforge::{diagnostic::pool::DiagnosticPoolProvider, error::{report::Report, FileforgeError}};
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
use fileforge::error::render::{buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT}, builtin::text::r#const::ConstText};
use fileforge_macros::text;
use crate::report::{render_with_context, CORRUPTED};

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

#[story("reader could not be acquired", BymlConstructionError::<StoryUserError, StoryStream>::ReaderAcquire(StoryUserError))]
#[story("node type failed to read", BymlConstructionError::<StoryUserError, StoryStream>::ReadType(BymlNodeDiscriminantsReadError::UnknownDiscriminant(0x42)))]
#[story("node type cannot be read dynamically", BymlConstructionError::<StoryUserError, StoryStream>::InvalidDynConstructable(BymlNodeDiscriminants::String))]
#[story("string table failed", BymlConstructionError::<StoryUserError, StoryStream>::StringTable(BymlStringTableNodeConstructableError::ReadCountError(read_exhausted::<intx::U24, StoryUserError>(dr!("data.byml" @ 0..34), 33, DiagnosticValue(34, None)))))]
pub enum BymlConstructionError<'pool, E, S: ReadableStream<Type = u8>> {
  ReaderAcquire(E),
  ReadType(BymlNodeDiscriminantsReadError<'pool, S>),
  InvalidDynConstructable(BymlNodeDiscriminants),
  StringTable(BymlStringTableNodeConstructableError<'pool, S>),
}

const MOVING_TO_NODE: ConstText = ConstText::new("This happened while moving to a node.", &REPORT_INFO_LINE_TEXT);

/// A node type's name with its article, such as "an array", for reports.
pub(crate) fn node_type_name(node_type: BymlNodeDiscriminants) -> &'static str {
  match node_type {
    BymlNodeDiscriminants::String => "a string",
    BymlNodeDiscriminants::BinaryData => "a binary data",
    BymlNodeDiscriminants::BinaryDataWithParameter => "a binary data (with parameter)",
    BymlNodeDiscriminants::Array => "an array",
    BymlNodeDiscriminants::Dictionary => "a dictionary",
    BymlNodeDiscriminants::StringTable => "a string table",
    BymlNodeDiscriminants::BinaryDataTable => "a binary data table",
    BymlNodeDiscriminants::Bool => "a boolean",
    BymlNodeDiscriminants::Integer32 => "a 32-bit integer",
    BymlNodeDiscriminants::Float32 => "a 32-bit float",
    BymlNodeDiscriminants::UnsignedInteger32 => "an unsigned 32-bit integer",
    BymlNodeDiscriminants::Integer64 => "a 64-bit integer",
    BymlNodeDiscriminants::UnsignedInteger64 => "an unsigned 64-bit integer",
    BymlNodeDiscriminants::Float64 => "a 64-bit float",
    BymlNodeDiscriminants::Null => "a null",
  }
}

impl<'pool, E, S: ReadableStream<Type = u8>> FileforgeError for BymlConstructionError<'pool, E, S>
where
  E: FileforgeError,
{
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::ReaderAcquire(error) => render_with_context(error, &MOVING_TO_NODE, provider, callback),
      Self::ReadType(error) => error.render_into_report(provider, callback),
      Self::InvalidDynConstructable(node_type) => {
        let node_type = node_type_name(*node_type);
        let found_text = text!([&REPORT_ERROR_TEXT] "An offset leads to {&node_type} node, but those are always stored in place, never behind an offset.");

        Report::new::<Self>(provider, &"Unexpected BYML node")
          .with_info_line(&found_text)
          .with_flag_line(&CORRUPTED)
          .apply(callback)
      }
      Self::StringTable(error) => error.render_into_report(provider, callback),
    }
  }
}

impl<'pool, E, S: ReadableStream<Type = u8>> From<BymlNodeDiscriminantsReadError<'pool, S>> for BymlConstructionError<'pool, E, S> {
  fn from(value: BymlNodeDiscriminantsReadError<'pool, S>) -> Self {
    Self::ReadType(value)
  }
}

impl<'pool, E, S: ReadableStream<Type = u8>> From<BymlStringTableNodeConstructableError<'pool, S>> for BymlConstructionError<'pool, E, S> {
  fn from(value: BymlStringTableNodeConstructableError<'pool, S>) -> Self {
    Self::StringTable(value)
  }
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
