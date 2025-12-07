use crate::{
  binary_reader::readable::{NoneArgument, Readable},
  error::FileforgeError,
  stream::{self, ReadableStream},
};

pub struct ArrayReadError<E: FileforgeError> {
  index: usize,
  error: E,
}

impl<E: FileforgeError> FileforgeError for ArrayReadError<E> {
  fn render_into_report<P: crate::diagnostic::pool::DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(crate::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    todo!()
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
