use core::convert::Infallible;

use crate::encodings::ascii::codepages::AsciiCodepage;

#[derive(Default, Debug)]
pub struct Iso8859_1;

impl AsciiCodepage for Iso8859_1 {
  type EncodeError = Infallible;
  type DecodeError = Infallible;

  fn from_byte(&self, byte: u8) -> Result<char, Self::EncodeError> {
    Ok(byte as char)
  }

  fn from_char(&self, c: char) -> Result<u8, Self::DecodeError> {
    Ok((c as u32).try_into().unwrap()) // TODO: Fix
  }
}
