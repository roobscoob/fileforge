pub mod codepages;
pub mod decoder;
pub mod encoder;

use fileforge::{encoding::Encoding, stream::ReadableStream};

use crate::encodings::ascii::{codepages::AsciiCodepage, decoder::AsciiDecoder, encoder::AsciiEncoder};

pub struct Ascii<Codepage: AsciiCodepage>(Codepage);

impl<Codepage: AsciiCodepage> Encoding for Ascii<Codepage> {
  type Encoder<S: ReadableStream<Type = u8>> = AsciiEncoder<Codepage, S>;
  type Decoder<S: ReadableStream<Type = char>> = AsciiDecoder<Codepage, S>;
}
