//! A codepage for stories whose conversions always fail, so its error variants can be shown.

use fileforge::storybook::StoryUserError;

use crate::encodings::ascii::codepages::AsciiCodepage;

#[derive(Default, Debug)]
pub struct StoryCodepage;

impl AsciiCodepage for StoryCodepage {
  type EncodeError = StoryUserError;
  type DecodeError = StoryUserError;

  fn from_byte(&self, _: u8) -> Result<char, Self::EncodeError> {
    Err(StoryUserError)
  }

  fn from_char(&self, _: char) -> Result<u8, Self::DecodeError> {
    Err(StoryUserError)
  }
}
