use fileforge_macros::{story, text};
use fileforge::{
  binary_reader::{error::SkipError, BinaryReader},
  stream::ReadableStream,
};
use fileforge::{diagnostic::pool::DiagnosticPoolProvider, error::{report::Report, FileforgeError}};

use crate::byml::{
  header::BymlHeader,
  node::{discriminant::BymlNodeDiscriminantVersionConfig, BymlAnyConstructable, BymlConstructionError, BymlNode, BymlNodeDiscriminants},
};
use fileforge::error::render::{
  buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT},
  builtin::{number::formatted_unsigned::FormattedUnsigned, text::r#const::ConstText},
};
use crate::report::{render_with_context, CORRUPTED};

pub mod header;
pub mod node;
pub mod readable;

pub struct Byml<'pool, S: ReadableStream<Type = u8>> {
  header: BymlHeader,
  reader: BinaryReader<'pool, S>,
}

#[story("out of bounds", {
  let error: IntoDataAtError<'_, StoryStream> = IntoDataAtError::OutOfBounds { offset: 0x8, header_size: 0x10 };
  error
})]
#[story("seek failed", IntoDataAtError::<StoryStream>::SeekError(fileforge::binary_reader::error::SkipError::OutOfBounds(fileforge::binary_reader::error::seek_out_of_bounds::SeekOutOfBounds {
  seek_offset: fileforge::binary_reader::error::common::SeekOffset::InBounds(64),
  provider_size: DiagnosticValue(32, None),
  container_dr: dr!("data.byml" @ 0..32),
})))]
pub enum IntoDataAtError<'pool, S: ReadableStream> {
  /// `offset` is before the end of the header, which is `header_size` bytes long.
  OutOfBounds { offset: u64, header_size: u64 },
  SeekError(SkipError<'pool, S::SkipError>),
}

const MOVING_TO_NODE_DATA: ConstText = ConstText::new("This happened while moving to a node's data.", &REPORT_INFO_LINE_TEXT);

impl<'pool, S: ReadableStream> FileforgeError for IntoDataAtError<'pool, S> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::OutOfBounds { offset, header_size } => {
        let offset = FormattedUnsigned::new(*offset as u128).base(16).uppercase().prefix("0x");
        let header_size_text = FormattedUnsigned::new(*header_size as u128).separator(3, ",");
        let header_size_hex = FormattedUnsigned::new(*header_size as u128).base(16).uppercase().prefix("0x");

        let offset_text = text!([&REPORT_ERROR_TEXT] "A node's offset is {&offset}, but the BYML header takes up the first {&header_size_text} ({&header_size_hex}) bytes.");

        Report::new::<Self>(provider, &"Node offset points into the header")
          .with_info_line(&offset_text)
          .with_flag_line(&CORRUPTED)
          .apply(callback)
      }
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
    let header_size = self.header.size();
    let offset = position.checked_sub(header_size).ok_or(IntoDataAtError::OutOfBounds { offset: position, header_size })?;
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
