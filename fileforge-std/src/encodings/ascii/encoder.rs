use fileforge_macros::story;
use fileforge::{
  diagnostic::pool::DiagnosticPoolProvider,
  encoding::Encoder,
  error::{
    render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::r#const::ConstText},
    report::Report,
    FileforgeError,
  },
  stream::{ReadableStream, StreamReadError, UserReadError},
};

use crate::encodings::ascii::codepages::AsciiCodepage;

pub struct AsciiEncoder<Codepage: AsciiCodepage, S: ReadableStream<Type = u8>>(Codepage, S);

#[story("stream failed", AsciiEncodeError::<StoryUserError, crate::encodings::ascii::codepages::iso_8859_1::Iso8859_1>::User(StoryUserError))]
#[story("codepage failed", AsciiEncodeError::<StoryUserError, crate::encodings::ascii::codepages::story::StoryCodepage>::Codepage(StoryUserError))]
#[derive(Debug)]
pub enum AsciiEncodeError<E: UserReadError, Codepage: AsciiCodepage> {
  User(E),
  Codepage(Codepage::EncodeError),
}

const READING_TEXT: ConstText = ConstText::new("This happened while encoding ASCII text.", &REPORT_INFO_LINE_TEXT);
const CONVERTING_CHARACTER: ConstText = ConstText::new("This happened while the ASCII codepage was converting a character.", &REPORT_INFO_LINE_TEXT);

impl<E: UserReadError, Codepage: AsciiCodepage> FileforgeError for AsciiEncodeError<E, Codepage> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, |report| report.with_info_line(&READING_TEXT).apply(callback)),
      Self::Codepage(error) => error.render_into_report(provider, |report| report.with_info_line(&CONVERTING_CHARACTER).apply(callback)),
    }
  }
}

impl<Codepage: AsciiCodepage, S: UserReadError> UserReadError for AsciiEncodeError<S, Codepage> {}

impl<Codepage: AsciiCodepage, S: ReadableStream<Type = u8>> Encoder<S> for AsciiEncoder<Codepage, S> {
  fn new(input: S) -> Self {
    AsciiEncoder(Codepage::default(), input)
  }
}

impl<Codepage: AsciiCodepage, S: ReadableStream<Type = u8>> ReadableStream for AsciiEncoder<Codepage, S> {
  type ReadError = AsciiEncodeError<S::ReadError, Codepage>;
  type SkipError = S::SkipError;
  type Type = char;

  fn len(&self) -> Option<u64> {
    self.1.len()
  }

  fn offset(&self) -> u64 {
    self.1.offset()
  }

  fn remaining(&self) -> Option<u64> {
    self.1.remaining()
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    match self
      .1
      .read(async |values: &[u8; SIZE]| {
        let mut string: [char; SIZE] = ['\x00'; SIZE];

        for (i, &item) in values.iter().enumerate() {
          string[i] = match item {
            c @ 0..128 => c as char,
            p @ 128.. => self.0.from_byte(p)?,
          }
        }

        Ok(reader(&string).await)
      })
      .await
    {
      Ok(Ok(v)) => Ok(v),
      Ok(Err(e)) => Err(StreamReadError::User(AsciiEncodeError::Codepage(e))),
      Err(StreamReadError::User(e)) => Err(StreamReadError::User(AsciiEncodeError::User(e))),
      Err(StreamReadError::StreamExhausted(e)) => Err(StreamReadError::StreamExhausted(e)),
    }
  }

  async fn skip(&mut self, size: u64) -> Result<(), fileforge::stream::StreamSkipError<Self::SkipError>> {
    self.1.skip(size).await
  }
}
