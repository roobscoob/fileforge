use fileforge_macros::{story, text};
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use crate::binary_reader::error::common::LOW_LEVEL_ERROR;
use crate::error::render::buffer::cell::tag::builtin::report::REPORT_ERROR_TEXT;
use crate::error::render::builtin::number::formatted_unsigned::FormattedUnsigned;
use super::user_rewind::UserRewindError;

#[story("stream failed", StreamRewindError::User(StoryUserError))]
#[story("rewind underflowed", StreamRewindError::<StoryUserError>::SeekPointUnderflowed { stream_length: 16, offset: 2, seek_backwards_distance: 6 })]
pub enum StreamRewindError<UserRewind: UserRewindError> {
  User(UserRewind),

  // ASSERT: offset - seek_backwards_distance < u64::MIN
  SeekPointUnderflowed { stream_length: u64, offset: u64, seek_backwards_distance: u64 },
}

impl<UserRewind: UserRewindError> StreamRewindError<UserRewind> {
  pub fn assert_relative_backwards(stream_length: u64, offset: u64, relative_backwards: u64) -> Result<u64, Self> {
    let seek_point = offset.checked_sub(relative_backwards).ok_or(Self::SeekPointUnderflowed {
      stream_length,
      offset,
      seek_backwards_distance: relative_backwards,
    })?;

    Ok(seek_point)
  }
}

impl<U: UserRewindError> FileforgeError for StreamRewindError<U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, callback),
      Self::SeekPointUnderflowed { stream_length: _, offset, seek_backwards_distance } => {
        let distance = FormattedUnsigned::new(*seek_backwards_distance as u128).separator(3, ",");
        let offset = FormattedUnsigned::new(*offset as u128).base(16).uppercase().prefix("0x");

        let rewind_text = text!(
          { *seek_backwards_distance == 1 }
            [&REPORT_ERROR_TEXT] "Rewinding 1 item from {&offset} goes before the start of the stream.",

          [&REPORT_ERROR_TEXT] "Rewinding {&distance} items from {&offset} goes before the start of the stream."
        );

        Report::new::<Self>(provider, &"Rewind underflowed")
          .with_info_line(&rewind_text)
          .with_flag_line(LOW_LEVEL_ERROR)
          .apply(callback)
      }
    }
  }
}
