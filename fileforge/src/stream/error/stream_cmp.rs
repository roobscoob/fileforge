use fileforge_macros::story;

use crate::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::r#const::ConstText},
    report::Report,
    FileforgeError,
  },
};

use super::user_read::UserReadError;

/// Reading one of two streams being compared failed. (Either running out isn't an error: it ends
/// that side of the comparison.)
#[story("the first stream failed", StreamCmpError::<StoryUserError, StoryUserError>::First(StoryUserError))]
#[story("the second stream failed", StreamCmpError::<StoryUserError, StoryUserError>::Second(StoryUserError))]
#[derive(Debug)]
pub enum StreamCmpError<First: UserReadError, Second: UserReadError> {
  First(First),
  Second(Second),
}

const READING_FIRST: ConstText = ConstText::new("This happened while reading the first of two streams being compared.", &REPORT_INFO_LINE_TEXT);
const READING_SECOND: ConstText = ConstText::new("This happened while reading the second of two streams being compared.", &REPORT_INFO_LINE_TEXT);

impl<First: UserReadError, Second: UserReadError> FileforgeError for StreamCmpError<First, Second> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::First(error) => error.render_into_report(provider, |report| report.with_info_line(&READING_FIRST).apply(callback)),
      Self::Second(error) => error.render_into_report(provider, |report| report.with_info_line(&READING_SECOND).apply(callback)),
    }
  }
}
