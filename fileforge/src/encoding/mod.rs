use crate::stream::ReadableStream;

pub trait Encoder<S: ReadableStream<Type = u8>>: ReadableStream<Type = char> {
  fn new(input: S) -> Self;
}

pub trait Decoder<S: ReadableStream<Type = char>>: ReadableStream<Type = u8> {
  fn new(input: S) -> Self;
}

pub trait Encoding {
  type Encoder<S: ReadableStream<Type = u8>>: Encoder<S>;
  type Decoder<S: ReadableStream<Type = char>>: Decoder<S>;
}
