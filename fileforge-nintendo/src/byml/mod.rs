use fileforge_macros::story;
use fileforge::{
  binary_reader::{error::SkipError, BinaryReader},
  stream::ReadableStream,
};
use fileforge::{diagnostic::pool::DiagnosticPoolProvider, error::{report::Report, FileforgeError}};

use crate::byml::{
  header::BymlHeader,
  node::{discriminant::BymlNodeDiscriminantVersionConfig, BymlAnyConstructable, BymlConstructionError, BymlNode, BymlNodeDiscriminants},
};
use fileforge::error::render::{buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT}, builtin::text::r#const::ConstText};
use crate::report::{render_with_context, CORRUPTED};

pub mod header;
pub mod node;
pub mod readable;

pub struct Byml<'pool, S: ReadableStream<Type = u8>> {
  header: BymlHeader,
  reader: BinaryReader<'pool, S>,
}

#[story("out of bounds", IntoDataAtError::<StoryStream>::OutOfBounds)]
#[story("seek failed", IntoDataAtError::<StoryStream>::SeekError(fileforge::binary_reader::error::SkipError::OutOfBounds(fileforge::binary_reader::error::seek_out_of_bounds::SeekOutOfBounds {
  seek_offset: fileforge::binary_reader::error::common::SeekOffset::InBounds(64),
  provider_size: DiagnosticValue(32, None),
  container_dr: dr!("data.byml" @ 0..32),
})))]
pub enum IntoDataAtError<'pool, S: ReadableStream> {
  OutOfBounds,
  SeekError(SkipError<'pool, S::SkipError>),
}

const OFFSET_INTO_HEADER: ConstText = ConstText::new("A node's offset points inside the BYML header, where no node data can be.", &REPORT_ERROR_TEXT);
const MOVING_TO_NODE_DATA: ConstText = ConstText::new("This happened while moving to a node's data.", &REPORT_INFO_LINE_TEXT);

impl<'pool, S: ReadableStream> FileforgeError for IntoDataAtError<'pool, S> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::OutOfBounds => Report::new::<Self>(provider, &"Node offset points into the header")
        .with_info_line(&OFFSET_INTO_HEADER)
        .with_flag_line(&CORRUPTED)
        .apply(callback),
      Self::SeekError(error) => render_with_context(error, &MOVING_TO_NODE_DATA, provider, callback),
    }
  }
}

impl<'pool, S: ReadableStream> From<SkipError<'pool, S::SkipError>> for IntoDataAtError<'pool, S> {
  fn from(value: SkipError<'pool, S::SkipError>) -> Self {
    Self::SeekError(value)
  }
}

impl<'pool, S: ReadableStream<Type = u8>> Byml<'pool, S> {
  fn version(&self) -> BymlNodeDiscriminantVersionConfig {
    BymlNodeDiscriminantVersionConfig {
      version_number: self.header.version(),
      feat_binary_data_table: self.header.config().feat_binary_data_table,
    }
  }

  async fn into_data_at(mut self, position: u64) -> Result<BinaryReader<'pool, S>, IntoDataAtError<'pool, S>> {
    let offset = position.checked_sub(self.header.size()).ok_or(IntoDataAtError::OutOfBounds)?;
    self.reader.skip(offset).await?;

    Ok(self.reader)
  }

  async fn into_node(self, discriminant: BymlNodeDiscriminants, value: u32) -> Result<BymlNode<'pool, S>, BymlConstructionError<'pool, IntoDataAtError<'pool, S>, S>> {
    BymlNode::construct(discriminant, value, self.version(), async move |offset| self.into_data_at(offset).await).await
  }

  async fn into_node_dyn(self, value: u32) -> Result<BymlNode<'pool, S>, BymlConstructionError<'pool, IntoDataAtError<'pool, S>, S>> {
    BymlNode::construct_dyn(value, self.version(), async move |offset| self.into_data_at(offset).await).await
  }

  pub async fn into_literal_table(self) -> Result<Option<BymlNode<'pool, S>>, BymlConstructionError<'pool, IntoDataAtError<'pool, S>, S>> {
    match self.header.string_table_offset() {
      Some(v) => Ok(Some(self.into_node_dyn(v.get()).await?)),
      None => Ok(None),
    }
  }

  pub async fn into_key_table(self) -> Result<Option<BymlNode<'pool, S>>, BymlConstructionError<'pool, IntoDataAtError<'pool, S>, S>> {
    match self.header.key_table_offset() {
      Some(v) => Ok(Some(self.into_node_dyn(v.get()).await?)),
      None => Ok(None),
    }
  }
}
