use fileforge_macros::{story, text};
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use crate::binary_reader::error::common::LOW_LEVEL_ERROR;
use crate::error::render::buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT};
use crate::error::render::builtin::number::formatted_unsigned::FormattedUnsigned;
use super::{stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, user_skip::UserSkipError};

#[story("stream failed", StreamSkipError::User(StoryUserError))]
#[story("out of bounds", StreamSkipError::<StoryUserError>::OutOfBounds(StreamSeekOutOfBoundsError { stream_length: 16, seek_point: 40 }))]
#[story("skip overflowed", StreamSkipError::<StoryUserError>::SeekPointOverflowed { stream_length: 16, offset: 8, seek_forwards_distance: u64::MAX })]
#[derive(Debug)]
pub enum StreamSkipError<UserSkip: UserSkipError> {
  User(UserSkip),
  OutOfBounds(StreamSeekOutOfBoundsError),

  // ASSERT: offset + seek_forwards_distance > u64::MAX
  SeekPointOverflowed { stream_length: u64, offset: u64, seek_forwards_distance: u64 },
}

impl<UserSkip: UserSkipError> From<StreamSeekOutOfBoundsError> for StreamSkipError<UserSkip> {
  fn from(value: StreamSeekOutOfBoundsError) -> Self {
    Self::OutOfBounds(value)
  }
}

impl<UserSkip: UserSkipError> From<UserSkip> for StreamSkipError<UserSkip> {
  fn from(value: UserSkip) -> Self {
    Self::User(value)
  }
}

impl<UserSkip: UserSkipError> StreamSkipError<UserSkip> {
  pub fn assert_relative_forwards(stream_length: u64, offset: u64, relative_forwards: u64) -> Result<u64, Self> {
    let seek_point = offset.checked_add(relative_forwards).ok_or(Self::SeekPointOverflowed {
      stream_length,
      offset,
      seek_forwards_distance: relative_forwards,
    })?;

    StreamSeekOutOfBoundsError::assert(stream_length, seek_point)?;

    Ok(seek_point)
  }
}

impl<U: UserSkipError> FileforgeError for StreamSkipError<U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, callback),
      Self::OutOfBounds(error) => error.render_into_report(provider, callback),
      Self::SeekPointOverflowed { stream_length, offset, seek_forwards_distance } => {
        let distance = FormattedUnsigned::new(*seek_forwards_distance as u128).base(16).uppercase().prefix("0x");
        let offset = FormattedUnsigned::new(*offset as u128).base(16).uppercase().prefix("0x");
        let stream_length = FormattedUnsigned::new(*stream_length as u128).separator(3, ",");

        let skip_text = text!([&REPORT_ERROR_TEXT] "Skipping {&distance} items from {&offset} goes past the largest possible position.");
        let length_text = text!([&REPORT_INFO_LINE_TEXT] "The stream is {&stream_length} items long.");

        Report::new::<Self>(provider, &"Skip overflowed")
          .with_info_line(&skip_text)
          .with_info_line(&length_text)
          .with_flag_line(LOW_LEVEL_ERROR)
          .apply(callback)
      }
    }
  }
}
