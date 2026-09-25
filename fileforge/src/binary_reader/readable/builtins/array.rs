use fileforge_macros::{story, text};
use crate::{
  binary_reader::readable::Readable,
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::number::formatted_unsigned::FormattedUnsigned},
    report::Report,
    FileforgeError,
  },
  stream::{self, ReadableStream},
};

#[story("element failed to read", ArrayReadError {
  index: 2,
  error: read_exhausted::<u32, StoryUserError>(dr!("save.bin" @ 0..10), 8, DiagnosticValue(10, None)),
})]
pub struct ArrayReadError<E: FileforgeError> {
  index: usize,
  error: E,
}

impl<E: FileforgeError> FileforgeError for ArrayReadError<E> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    self.error.render_into_report(provider, |report| {
      let index = FormattedUnsigned::new(self.index as u128).separator(3, ",");
      let context = text!([&REPORT_INFO_LINE_TEXT] "This happened while reading the item at index {&index} of an array.");

      report.with_info_line(&context).apply(callback)
    });
  }
}

impl<E: FileforgeError> stream::UserReadError for ArrayReadError<E> {}

impl<'pool, S: ReadableStream<Type = u8>, T: Readable<'pool, S>, const N: usize> Readable<'pool, S> for [T; N]
{
  type Error = ArrayReadError<T::Error>;

  type Argument = [T::Argument; N];

  async fn read(reader: &mut crate::binary_reader::BinaryReader<'pool, S>, arguments: Self::Argument) -> Result<Self, Self::Error> {
    let mut vec = heapless::Vec::<T, N>::new();
    for (index, argument) in arguments.into_iter().enumerate() {
      let item = reader.read_with(argument).await.map_err(|error| ArrayReadError { index, error })?;

      vec.push(item).map_err(|_| ()).unwrap();
    }

    Ok(vec.into_array().map_err(|_| ()).unwrap())
  }
}
