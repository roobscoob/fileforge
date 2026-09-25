//! Word wrapping for text made of literal pieces and interpolated renderables.
//!
//! Text is wrapped a *word* at a time, where a word runs up to the next whitespace and may be
//! made of several pieces: in `"(0x{&offset})."`, the `(0x`, the number and the `).` are one word.
//! A word that doesn't fit on the current line starts the next one, whole, and the space before
//! it is dropped. A word wider than a whole line is broken wherever it overflows.

use unicode_segmentation::{UWordBounds, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

use crate::error::render::{
  buffer::{canvas::RenderBufferCanvas, cell::tag::CellTag, RenderBuffer},
  position::RenderPosition,
  r#trait::renderable::Renderable,
};

use super::TextSegment;

enum Token<'l, 't> {
  Word(&'l str, Option<&'t dyn CellTag>),
  Space(&'l str, Option<&'t dyn CellTag>),
  Newline,
  Renderable(&'l dyn Renderable<'t>),
}

/// Walks the segments' words, spaces, line breaks and renderables in order. Cloning it gives a
/// lookahead that doesn't disturb the original.
#[derive(Clone)]
struct Tokens<'s, 'l, 't> {
  segments: &'s [TextSegment<'l, 't>],
  next_segment: usize,
  chunks: Option<(UWordBounds<'l>, Option<&'t dyn CellTag>)>,
}

impl<'s, 'l, 't> Iterator for Tokens<'s, 'l, 't> {
  type Item = Token<'l, 't>;

  fn next(&mut self) -> Option<Token<'l, 't>> {
    loop {
      if let Some((chunks, tag)) = &mut self.chunks {
        if let Some(chunk) = chunks.next() {
          let tag = *tag;

          return Some(if chunk == "\n" || chunk == "\r\n" {
            Token::Newline
          } else if chunk.chars().all(char::is_whitespace) {
            Token::Space(chunk, tag)
          } else {
            Token::Word(chunk, tag)
          });
        }

        self.chunks = None;
      }

      let segment = self.segments.get(self.next_segment)?;
      self.next_segment += 1;

      match segment {
        TextSegment::Renderable(renderable) => return Some(Token::Renderable(*renderable)),
        TextSegment::Segment(text, tag) => self.chunks = Some((text.split_word_bounds(), *tag)),
      }
    }
  }
}

/// The width of a renderable that fits on one line, or `None` if it spans several.
fn measure<'t>(renderable: &dyn Renderable<'t>) -> Option<usize> {
  let mut dry = RenderBuffer::dry();
  let summary = dry.canvas_at(RenderPosition::zero()).write(renderable).ok()?;

  (summary.get_line_height() == 1).then(|| summary.end_position.column())
}

/// The width of the word starting at `tokens`, or `None` if part of it spans several lines.
fn word_width<'t>(mut tokens: Tokens<'_, '_, 't>) -> Option<usize> {
  let mut width = 0;

  loop {
    match tokens.next() {
      Some(Token::Word(text, _)) => width += text.width(),
      Some(Token::Renderable(renderable)) => width += measure(renderable)?,
      Some(Token::Space(..) | Token::Newline) | None => return Some(width),
    }
  }
}

fn write_str<'t>(canvas: &mut RenderBufferCanvas<'_, '_, 't>, text: &str, tag: Option<&'t dyn CellTag>, start_column: usize) {
  for grapheme in text.graphemes(true) {
    let fits = match tag {
      Some(tag) => canvas.set_tagged_char(grapheme, tag),
      None => canvas.set_char(grapheme),
    };

    // Only a word wider than a whole line gets here: break it where it overflows.
    if !fits {
      canvas.cursor_down().set_column(start_column);

      match tag {
        Some(tag) => canvas.set_tagged_char(grapheme, tag),
        None => canvas.set_char(grapheme),
      };
    }
  }
}

/// Renders `segments` from the canvas' position, wrapping back to its column.
pub(crate) fn render_wrapped<'t>(segments: &[TextSegment<'_, 't>], canvas: &mut RenderBufferCanvas<'_, '_, 't>) -> Result<(), ()> {
  let start_column = canvas.get_position().column();
  let line_width = canvas.buffer.width();

  let mut tokens = Tokens {
    segments,
    next_segment: 0,
    chunks: None,
  };

  // Whitespace is held back until the next word: if that word wraps, the space is dropped.
  let mut pending_space = None;

  loop {
    let mut lookahead = tokens.clone();

    match lookahead.next() {
      None => break,

      Some(Token::Newline) => {
        tokens = lookahead;
        pending_space = None;
        canvas.cursor_down().set_column(start_column);
      }

      Some(Token::Space(text, tag)) => {
        tokens = lookahead;

        if let Some((previous, previous_tag)) = pending_space.replace((text, tag)) {
          write_str(canvas, previous, previous_tag, start_column);
        }
      }

      Some(Token::Word(..) | Token::Renderable(..)) => {
        let space_width = pending_space.map(|(text, _): (&str, _)| text.width()).unwrap_or(0);
        let column = canvas.get_position().column();

        let fits = match word_width(tokens.clone()) {
          Some(width) => column + space_width + width <= line_width,
          None => true,
        };

        if !fits && column > start_column {
          canvas.cursor_down().set_column(start_column);
          pending_space = None;
        }

        if let Some((text, tag)) = pending_space.take() {
          write_str(canvas, text, tag, start_column);
        }

        // Write the word, piece by piece.
        loop {
          let mut lookahead = tokens.clone();

          match lookahead.next() {
            Some(Token::Word(text, tag)) => write_str(canvas, text, tag, start_column),
            Some(Token::Renderable(renderable)) => {
              canvas.write(renderable)?;
            }
            _ => break,
          }

          tokens = lookahead;
        }
      }
    }
  }

  if let Some((text, tag)) = pending_space {
    if canvas.get_position().column() + text.width() <= line_width {
      write_str(canvas, text, tag, start_column);
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::{string::String, vec, vec::Vec};

  use crate::error::render::{
    buffer::{
      cell::{tag::context::RenderMode, RenderBufferCell},
      RenderBuffer,
    },
    builtin::{number::formatted_unsigned::FormattedUnsigned, text::Text},
    position::RenderPosition,
    r#trait::renderable::Renderable,
  };

  fn render(renderable: &dyn Renderable<'static>, width: usize) -> String {
    let mut cells = vec![RenderBufferCell::default(); width * 10];
    let mut buffer = RenderBuffer::new(&mut cells, width, 0);
    let height = buffer.canvas_at(RenderPosition::zero()).write(renderable).unwrap().get_line_height();

    let mut out = String::new();
    buffer.flush_into(&mut out, RenderMode::PlainText).unwrap();
    out.lines().take(height).map(str::trim_end).collect::<Vec<_>>().join("\n")
  }

  #[test]
  fn a_value_that_does_not_fit_moves_to_the_next_line() {
    let four = FormattedUnsigned::new(4);
    let text = Text::of("It needs ").with(&four).push(" bytes");

    assert_eq!(render(&text, 9), "It needs\n4 bytes");
  }

  #[test]
  fn punctuation_stays_with_its_word() {
    assert_eq!(render(&"aaaa bbbbb.", 10), "aaaa\nbbbbb.");
    assert_eq!(render(&"the container's length, of 8 bytes", 22), "the container's\nlength, of 8 bytes");
  }

  #[test]
  fn a_word_made_of_several_pieces_wraps_whole() {
    let hex = FormattedUnsigned::new(0x1F).base(16).uppercase();
    let text = Text::of("abcdef (0x").with(&hex).push(").");

    assert_eq!(render(&text, 12), "abcdef\n(0x1F).");
  }

  #[test]
  fn wrapped_lines_do_not_start_with_a_space() {
    assert_eq!(render(&"one two three four", 8), "one two\nthree\nfour");
  }

  #[test]
  fn a_word_longer_than_a_line_is_broken() {
    assert_eq!(render(&"abcdefgh", 5), "abcde\nfgh");
  }

  #[test]
  fn line_breaks_are_kept() {
    assert_eq!(render(&"ab\ncd", 10), "ab\ncd");
  }
}
