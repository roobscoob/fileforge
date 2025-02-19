use fileforge_macros::text;

use crate::{error::{render::{buffer::cell::tag::builtin::report::{ReportFlagLineText, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT}, builtin::text::{self, r#const::ConstText, Text}}, report::Report, FileforgeError}, stream::error::{stream_read::StreamReadError, user_read::UserReadError}};

use super::exhausted::ReaderExhaustedError;

pub enum GetPrimitiveError<'pool, const NODE_NAME_SIZE: usize, U: UserReadError<'pool, NODE_NAME_SIZE>> {
  ReaderExhausted(&'static str, ReaderExhaustedError<'pool, NODE_NAME_SIZE>),
  User(&'static str, U),
}

impl<'pool, const NODE_NAME_SIZE: usize, U: UserReadError<'pool, NODE_NAME_SIZE>> FileforgeError<'pool, NODE_NAME_SIZE> for GetPrimitiveError<'pool, NODE_NAME_SIZE, U> {
  fn render_into_report(&self, mut callback: impl for<'a, 'b> FnMut(crate::error::report::Report<'a, 'b, 'pool, NODE_NAME_SIZE>) -> ()) {
    let text = *match self {
      Self::ReaderExhausted(text, ..) => text,
      Self::User(text, ..) => text,
    };

    match self {
      Self::ReaderExhausted(s, e) => {
        e.render_into_report(|report| {
          let t = Text::of(&text);
          let t = text!([&REPORT_INFO_LINE_TEXT] "This error originated while attempting to read {&t}");

          callback(report.with_info_line(&t).unwrap());
        });
      },
      Self::User(s, e) => {
        e.render_into_report(|report| {
          let t = Text::of(&text);
          let t = text!([&REPORT_INFO_LINE_TEXT] "This error originated while attempting to read {&t}");

          callback(report.with_info_line(&t).unwrap());
        });
      }
    }
  }
}