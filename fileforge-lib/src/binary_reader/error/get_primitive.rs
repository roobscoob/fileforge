use fileforge_macros::text;

use crate::{diagnostic::pool::DiagnosticPoolProvider, error::{render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::Text}, report::Report, FileforgeError}, stream::error::user_read::UserReadError};

use super::exhausted::ReaderExhaustedError;

pub enum GetPrimitiveError<'pool, U: UserReadError> {
  ReaderExhausted(&'static str, ReaderExhaustedError<'pool, true>),
  User(&'static str, U),
}

impl<'pool, U: UserReadError> FileforgeError for GetPrimitiveError<'pool, U> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(&self, provider: &'pool_ref P, mut callback: impl for<'tag, 'b, 'p2> FnMut(Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> ()) {
    let text = *match self {
      Self::ReaderExhausted(text, ..) => text,
      Self::User(text, ..) => text,
    };

    match self {
      Self::ReaderExhausted(_, e) => {
        e.render_into_report(provider, |report| {
          let t = Text::of(&text);
          let t = text!([&REPORT_INFO_LINE_TEXT] "This error originated while attempting to read {&t}");

          callback(report.with_info_line(&t).unwrap());
        });
      },
      Self::User(_, e) => {
        e.render_into_report(provider, |report| {
          let t = Text::of(&text);
          let t = text!([&REPORT_INFO_LINE_TEXT] "This error originated while attempting to read {&t}");

          callback(report.with_info_line(&t).unwrap());
        });
      }
    }
  }
}