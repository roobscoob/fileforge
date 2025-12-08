use fileforge::error::FileforgeError;

pub mod iso_8859_1;

pub trait AsciiCodepage: Default {
  type EncodeError: FileforgeError;
  type DecodeError: FileforgeError;

  fn from_byte(&self, byte: u8) -> Result<char, Self::EncodeError>;
  fn from_char(&self, char: char) -> Result<u8, Self::DecodeError>;
}
