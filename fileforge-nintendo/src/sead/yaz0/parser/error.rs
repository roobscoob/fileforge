use fileforge_macros::story;
use fileforge::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::FileforgeError,
  stream::error::{
    stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_read::StreamReadError, stream_restore::StreamRestoreError, stream_skip::StreamSkipError,
    user_mutate::UserMutateError, user_overwrite::UserOverwriteError, user_read::UserReadError, user_restore::UserRestoreError, user_skip::UserSkipError,
  },
};
use fileforge::error::render::{buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT}, builtin::text::r#const::ConstText};
use fileforge_macros::text;
use fileforge::error::report::Report;
use crate::report::{TRUNCATED};
use crate::report::{render_with_context};

#[derive(Debug, Clone, Copy)]
pub enum Component {
  Header,
  Literal,
  SequenceHeader,
  SmallSequenceTail,
  LargeSequenceTail,
}

#[story("stream ran out", Yaz0ParserError::<StoryUserError>::ReadFailed(Component::SequenceHeader, StreamReadError::StreamExhausted(fileforge::stream::error::stream_exhausted::StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 })))]
#[story("stream failed", Yaz0ParserError::ReadError(Component::Header, StoryUserError))]
#[derive(Debug)]
pub enum Yaz0ParserError<SURE: UserReadError> {
  ReadFailed(Component, StreamReadError<SURE>),
  ReadError(Component, SURE),
}

#[story("skipped past the end", Yaz0ParserSkipError::<StoryUserError, StoryUserError>::SkipFailed(Component::Literal, StreamSkipError::OutOfBounds(fileforge::stream::error::stream_seek_out_of_bounds::StreamSeekOutOfBoundsError { stream_length: 16, seek_point: 17 })))]
#[story("stream ran out", Yaz0ParserSkipError::<StoryUserError, StoryUserError>::ReadFailed(Component::SequenceHeader, StreamReadError::StreamExhausted(fileforge::stream::error::stream_exhausted::StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 })))]
#[story("stream failed", Yaz0ParserSkipError::<StoryUserError, StoryUserError>::ReadError(Component::Header, StoryUserError))]
#[story("skip failed", Yaz0ParserSkipError::<StoryUserError, StoryUserError>::SkipFailed(Component::Literal, StreamSkipError::User(StoryUserError)))]
#[derive(Debug)]
pub enum Yaz0ParserSkipError<SURE: UserReadError, SUSE: UserSkipError> {
  ReadFailed(Component, StreamReadError<SURE>),
  ReadError(Component, SURE),
  SkipFailed(Component, StreamSkipError<SUSE>),
}

#[story("read failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::ReadFailed(Yaz0ParserError::ReadError(Component::Header, StoryUserError)))]
#[story("restore failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::RestoreFailed(StreamRestoreError::CannotRestoreForwards))]
#[story("skipping the header failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::SkipHeaderFailed(StreamSkipError::User(StoryUserError)))]
#[story("skipping an operation failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::SkipOperationFailed(StreamSkipError::User(StoryUserError)))]
#[story("mutating the header failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::MutateHeaderFailed(StreamMutateError::User(StoryUserError)))]
#[story("overwriting a literal failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::OverwriteLiteralFailed(StreamOverwriteError::User(StoryUserError)))]
#[story("overwriting a short readback failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::OverwriteShortReadbackFailed(StreamOverwriteError::User(StoryUserError)))]
#[story("shrinkage blocked", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::ShrinkageBlocked)]
#[story("removing the header failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::RemoveHeaderFailed(StreamOverwriteError::User(StoryUserError)))]
#[story("removing a readback failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::RemoveReadbackFailed(StreamOverwriteError::User(StoryUserError)))]
#[story("creating the header failed", Yaz0ParserMutateError::<StoryUserError, StoryUserError, StoryUserError, StoryUserError, StoryUserError>::CreateHeaderFailed(StreamOverwriteError::User(StoryUserError)))]
#[derive(Debug)]
pub enum Yaz0ParserMutateError<SURE: UserReadError, SUREE: UserRestoreError, SUSE: UserSkipError, SUOE: UserOverwriteError, SUME: UserMutateError> {
  ReadFailed(Yaz0ParserError<SURE>),
  RestoreFailed(StreamRestoreError<SUREE>),
  SkipHeaderFailed(StreamSkipError<SUSE>),
  SkipOperationFailed(StreamSkipError<SUSE>),
  MutateHeaderFailed(StreamMutateError<SUME>),
  OverwriteLiteralFailed(StreamOverwriteError<SUOE>),
  OverwriteShortReadbackFailed(StreamOverwriteError<SUOE>),
  ShrinkageBlocked,
  RemoveHeaderFailed(StreamOverwriteError<SUOE>),
  RemoveReadbackFailed(StreamOverwriteError<SUOE>),
  CreateHeaderFailed(StreamOverwriteError<SUOE>),
}

impl<SURE: UserReadError> UserReadError for Yaz0ParserError<SURE> {}
impl<SURE: UserReadError, SUSE: UserSkipError> UserSkipError for Yaz0ParserSkipError<SURE, SUSE> {}
impl<SURE: UserReadError, SUREE: UserRestoreError, SUSE: UserSkipError, SUOE: UserOverwriteError, SUME: UserMutateError> UserMutateError for Yaz0ParserMutateError<SURE, SUREE, SUSE, SUOE, SUME> {}
impl<SURE: UserReadError, SUREE: UserRestoreError, SUSE: UserSkipError, SUOE: UserOverwriteError, SUME: UserMutateError> UserOverwriteError for Yaz0ParserMutateError<SURE, SUREE, SUSE, SUOE, SUME> {}

impl Component {
  /// What the component is, with its article, for reports.
  pub fn description(&self) -> &'static str {
    match self {
      Self::Header => "a group header",
      Self::Literal => "a literal byte",
      Self::SequenceHeader | Self::SmallSequenceTail => "a back-reference",
      Self::LargeSequenceTail => "a long back-reference's extra length byte",
    }
  }
}

/// Reports compressed data that ends partway through `component`.
fn render_truncated<E, P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
  component: Component,
  provider: P,
  callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
) {
  let component = component.description();
  let ends_text = text!([&REPORT_ERROR_TEXT] "The compressed data ends where {&component} should be.");

  Report::new::<E>(provider, &"Truncated Yaz0 data").with_info_line(&ends_text).with_flag_line(&TRUNCATED).apply(callback)
}


impl<SURE: UserReadError> FileforgeError for Yaz0ParserError<SURE> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    match self {
      Self::ReadFailed(component, StreamReadError::StreamExhausted(_)) => render_truncated::<Self, P, ITEM_NAME_SIZE>(*component, provider, callback),
      Self::ReadFailed(component, StreamReadError::User(error)) | Self::ReadError(component, error) => {
        let component = component.description();

        error.render_into_report(provider, move |report| {
          let context = text!([&REPORT_INFO_LINE_TEXT] "This happened while reading {&component} of the compressed data.");
          report.with_info_line(&context).apply(callback)
        })
      }
    }
  }
}

impl<SURE: UserReadError, SUSE: UserSkipError> FileforgeError for Yaz0ParserSkipError<SURE, SUSE> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    match self {
      Self::ReadFailed(component, StreamReadError::StreamExhausted(_)) => render_truncated::<Self, P, ITEM_NAME_SIZE>(*component, provider, callback),
      Self::ReadFailed(component, StreamReadError::User(error)) | Self::ReadError(component, error) => {
        let component = component.description();

        error.render_into_report(provider, move |report| {
          let context = text!([&REPORT_INFO_LINE_TEXT] "This happened while reading {&component} of the compressed data.");
          report.with_info_line(&context).apply(callback)
        })
      }
      Self::SkipFailed(component, StreamSkipError::OutOfBounds(_)) => render_truncated::<Self, P, ITEM_NAME_SIZE>(*component, provider, callback),
      Self::SkipFailed(component, error) => {
        let component = component.description();

        error.render_into_report(provider, move |report| {
          let context = text!([&REPORT_INFO_LINE_TEXT] "This happened while skipping over {&component} of the compressed data.");
          report.with_info_line(&context).apply(callback)
        })
      }
    }
  }
}

const RETURNING_TO_BLOCK: ConstText = ConstText::new("This happened while returning to the start of a block of Yaz0-compressed data to rewrite it.", &REPORT_INFO_LINE_TEXT);
const SKIPPING_GROUP_HEADER: ConstText = ConstText::new("This happened while skipping over a group header in Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const SKIPPING_OPERATION: ConstText = ConstText::new("This happened while skipping over an unchanged operation in Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const UPDATING_GROUP_HEADER: ConstText = ConstText::new("This happened while updating a group header in Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const WRITING_LITERAL: ConstText = ConstText::new("This happened while writing a literal byte into Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const WRITING_BACK_REFERENCE: ConstText = ConstText::new("This happened while writing a back-reference into Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const REMOVING_GROUP_HEADER: ConstText = ConstText::new("This happened while removing a group header from Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const REMOVING_BACK_REFERENCE: ConstText = ConstText::new("This happened while removing a back-reference from Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const ADDING_GROUP_HEADER: ConstText = ConstText::new("This happened while adding a group header to Yaz0-compressed data.", &REPORT_INFO_LINE_TEXT);
const SHRINKAGE_BLOCKED: ConstText = ConstText::new("The new data needs fewer operations than the old. That's only supported at the end of the compressed data.", &REPORT_ERROR_TEXT);

impl<SURE: UserReadError, SUREE: UserRestoreError, SUSE: UserSkipError, SUOE: UserOverwriteError, SUME: UserMutateError> FileforgeError for Yaz0ParserMutateError<SURE, SUREE, SUSE, SUOE, SUME> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    match self {
      Self::ReadFailed(error) => error.render_into_report(provider, callback),
      Self::RestoreFailed(error) => render_with_context(error, &RETURNING_TO_BLOCK, provider, callback),
      Self::SkipHeaderFailed(error) => render_with_context(error, &SKIPPING_GROUP_HEADER, provider, callback),
      Self::SkipOperationFailed(error) => render_with_context(error, &SKIPPING_OPERATION, provider, callback),
      Self::MutateHeaderFailed(error) => render_with_context(error, &UPDATING_GROUP_HEADER, provider, callback),
      Self::OverwriteLiteralFailed(error) => render_with_context(error, &WRITING_LITERAL, provider, callback),
      Self::OverwriteShortReadbackFailed(error) => render_with_context(error, &WRITING_BACK_REFERENCE, provider, callback),
      Self::ShrinkageBlocked => Report::new::<Self>(provider, &"Can't shrink compressed data here")
        .with_info_line(&SHRINKAGE_BLOCKED)
        .apply(callback),
      Self::RemoveHeaderFailed(error) => render_with_context(error, &REMOVING_GROUP_HEADER, provider, callback),
      Self::RemoveReadbackFailed(error) => render_with_context(error, &REMOVING_BACK_REFERENCE, provider, callback),
      Self::CreateHeaderFailed(error) => render_with_context(error, &ADDING_GROUP_HEADER, provider, callback),
    }
  }
}
