use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const GRAPHEME_WIDTH: usize = 8;

#[derive(Clone, Copy, Default)]
pub struct Grapheme {
  contents: [u8; GRAPHEME_WIDTH],
  /**
   * Heavy optimizations here :3
   * the MSB contains a did_overflow tag to display the grapheme overflow error
   * the 7 LSBs contain the length of contents
   */
  content_tag: u8,
}

impl Grapheme {
  pub fn from_str(str: &str) -> Self {
    str
      .graphemes(true)
      .next()
      .map(|grapheme| {
        let mut safe_length = 0;

        for (index, c) in grapheme.char_indices() {
          if (index + c.len_utf8()) > GRAPHEME_WIDTH {
            break;
          }

          safe_length = index + c.len_utf8();
        }

        let mut contents: [u8; GRAPHEME_WIDTH] = [0; GRAPHEME_WIDTH];

        contents[0..safe_length].copy_from_slice(grapheme[0..safe_length].as_bytes());

        let did_overflow = if safe_length > grapheme.as_bytes().len() {
          1u8
        } else {
          0u8
        };

        let content_tag = (safe_length & 0b0111_1111) as u8 | (did_overflow << 7);

        Grapheme {
          contents,
          content_tag,
        }
      })
      .unwrap_or(Grapheme {
        contents: [0; GRAPHEME_WIDTH],
        content_tag: 0,
      })
  }

  pub fn grapheme_iter(string: &str) -> impl Iterator<Item = Self> + '_ {
    string
      .graphemes(true)
      .map(|grapheme| Self::from_str(grapheme))
  }

  pub fn len(&self) -> usize { (self.content_tag & 0b0111_1111) as usize }

  pub fn did_overflow(&self) -> bool { (self.content_tag & 0b1000_0000) != 0 }

  pub fn as_str(&self) -> &str {
    unsafe { core::str::from_utf8_unchecked(&self.contents[0..self.len()]) }
  }

  pub fn is_empty(&self) -> bool { self.content_tag == 0 }

  pub fn width(&self) -> usize { self.as_str().width() }
}
