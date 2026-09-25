use fileforge_macros::story;
use fileforge::binary_reader::primitive::numeric::u24;
use fileforge::binary_reader::PrimitiveReader;
use fileforge::stream::builtin::read_until::ReadUntil;
use fileforge::stream::extensions::readable::ReadableStreamExt;
use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError, SkipError},
    readable::builtins::{
      array::ArrayReadError,
      contiugous::{Contiguous, ContiguousSkipError},
    },
    BinaryReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::{ReadableStream, StreamReadError, StreamSkipError, SINGLE},
  ResultIgnoreExt,
};
use fileforge::{diagnostic::pool::DiagnosticPoolProvider, error::{report::Report, FileforgeError}};

use crate::byml::node::BymlDynConstructable;
use fileforge::error::render::{buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT}, builtin::text::r#const::ConstText};
use crate::report::{render_field_read, render_with_context, Field, CORRUPTED};

pub struct BymlStringTableNode<'pool, S: ReadableStream<Type = u8>> {
  count: u32,
  reader: BinaryReader<'pool, S>,
}

#[story("file ends inside the string count", BymlStringTableNodeConstructableError::<StoryStream>::ReadCountError(read_exhausted::<intx::U24, StoryUserError>(dr!("data.byml" @ 0..34), 33, DiagnosticValue(34, None))))]
pub enum BymlStringTableNodeConstructableError<'pool, S: ReadableStream<Type = u8>> {
  ReadCountError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>),
}

impl<'pool, S: ReadableStream<Type = u8>> FileforgeError for BymlStringTableNodeConstructableError<'pool, S> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::ReadCountError(read) => render_field_read::<Self, S::ReadError, P, ITEM_NAME_SIZE>(
        read,
        Field { structure: "BYML string table", name: "string count", kind: "a u24" },
        provider,
        callback,
      ),
    }
  }
}

impl<'pool, S: ReadableStream<Type = u8>> BymlDynConstructable<'pool, S> for BymlStringTableNode<'pool, S> {
  type Error = BymlStringTableNodeConstructableError<'pool, S>;

  async fn construct_dyn(mut reader: BinaryReader<'pool, S>) -> Result<Self, Self::Error> {
    Ok(BymlStringTableNode {
      count: reader.get::<u24>().await.map_err(BymlStringTableNodeConstructableError::ReadCountError)?.into(),
      reader,
    })
  }
}

#[story("failed to skip to the index", BymlStringTableNodeIntoStringError::<StoryStream>::FailedToSkipToIndex(fileforge::stream::error::stream_skip::StreamSkipError::User(fileforge::binary_reader::readable::builtins::contiugous::ContiguousSkipError::Overflowed { index: 0, count: u64::MAX, item_size: Some(4) })))]
#[story("failed to read the offset", BymlStringTableNodeIntoStringError::<StoryStream>::FailedToReadOffset(fileforge::stream::error::stream_read::StreamReadError::StreamExhausted(fileforge::stream::error::stream_exhausted::StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 })))]
#[story("failed to consume the address table", BymlStringTableNodeIntoStringError::<StoryStream>::FailedToConsumeAddressTable(fileforge::stream::error::stream_skip::StreamSkipError::User(fileforge::binary_reader::readable::builtins::contiugous::ContiguousSkipError::Overflowed { index: 3, count: u64::MAX, item_size: Some(4) })))]
#[story("offset out of bounds", BymlStringTableNodeIntoStringError::<StoryStream>::OobOffset)]
#[story("failed to skip to the string", BymlStringTableNodeIntoStringError::<StoryStream>::FailedToSkipToString(fileforge::binary_reader::error::SkipError::User(StoryUserError)))]
pub enum BymlStringTableNodeIntoStringError<'pool, S: ReadableStream<Type = u8>> {
  FailedToSkipToIndex(StreamSkipError<ContiguousSkipError<'pool, S::SkipError, Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>>>),
  FailedToReadOffset(StreamReadError<ArrayReadError<Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>>>),
  FailedToConsumeAddressTable(StreamSkipError<ContiguousSkipError<'pool, S::SkipError, Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>>>),
  OobOffset,
  FailedToSkipToString(SkipError<'pool, S::SkipError>),
}

const FINDING_STRING: ConstText = ConstText::new("This happened while finding a string's entry in a BYML string table.", &REPORT_INFO_LINE_TEXT);
const READING_STRING_OFFSET: ConstText = ConstText::new("This happened while reading a string's offset from a BYML string table.", &REPORT_INFO_LINE_TEXT);
const SKIPPING_STRING_OFFSETS: ConstText = ConstText::new("This happened while skipping the rest of a BYML string table's offsets.", &REPORT_INFO_LINE_TEXT);
const MOVING_TO_STRING: ConstText = ConstText::new("This happened while moving to a string in a BYML string table.", &REPORT_INFO_LINE_TEXT);
const OFFSET_BEFORE_STRINGS: ConstText = ConstText::new("A string's offset points into the string table's list of offsets instead of at a string.", &REPORT_ERROR_TEXT);

impl<'pool, S: ReadableStream<Type = u8>> FileforgeError for BymlStringTableNodeIntoStringError<'pool, S> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::FailedToSkipToIndex(error) => render_with_context(error, &FINDING_STRING, provider, callback),
      Self::FailedToReadOffset(error) => render_with_context(error, &READING_STRING_OFFSET, provider, callback),
      Self::FailedToConsumeAddressTable(error) => render_with_context(error, &SKIPPING_STRING_OFFSETS, provider, callback),
      Self::OobOffset => Report::new::<Self>(provider, &"String offset out of range")
        .with_info_line(&OFFSET_BEFORE_STRINGS)
        .with_flag_line(&CORRUPTED)
        .apply(callback),
      Self::FailedToSkipToString(error) => render_with_context(error, &MOVING_TO_STRING, provider, callback),
    }
  }
}

impl<'pool, S: ReadableStream<Type = u8>> From<SkipError<'pool, S::SkipError>> for BymlStringTableNodeIntoStringError<'pool, S> {
  fn from(value: SkipError<'pool, S::SkipError>) -> Self {
    Self::FailedToSkipToString(value)
  }
}

impl<'pool, S: ReadableStream<Type = u8>> BymlStringTableNode<'pool, S> {
  fn address_table_length(&self) -> u64 {
    self.count as u64 + 1
  }

  fn node_offset_to_data_offset(&self, node_offset: u32) -> Option<u32> {
    let delta = 4 + (self.address_table_length() * 4);

    node_offset.checked_sub(delta.try_into().unwrap_or(u32::MAX))
  }

  pub async fn into_string(mut self, index: u32) -> Result<ReadUntil<S>, BymlStringTableNodeIntoStringError<'pool, S>> {
    let address_table_len = self.address_table_length();
    let mut address_table = self.reader.read_ref_with::<Contiguous<_, u32, _>>(|_| {}).await.ignore();

    address_table.skip(index as u64).await.map_err(BymlStringTableNodeIntoStringError::FailedToSkipToIndex)?;

    let node_offset = address_table.read(SINGLE).await.map_err(BymlStringTableNodeIntoStringError::FailedToReadOffset)?;

    address_table.finish(address_table_len).await.map_err(BymlStringTableNodeIntoStringError::FailedToConsumeAddressTable)?;

    let data_offset = self.node_offset_to_data_offset(node_offset).ok_or(BymlStringTableNodeIntoStringError::OobOffset)?;

    self.reader.skip(data_offset as u64).await?;

    Ok(self.reader.into_stream().read_until(0))
  }
}
