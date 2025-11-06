use core::fmt::Write;

use crate::error::render::grapheme::Grapheme;

use self::context::{CellTagContext, RenderMode};

pub mod builtin;
pub mod context;

pub trait CellTag {
  fn render_into(
    &self,
    writable: &mut dyn Write,
    grapheme: Grapheme,
    context: CellTagContext,
  ) -> Result<(), core::fmt::Error> {
    match context.mode {
      RenderMode::PlainText => writable.write_str(grapheme.as_str())?,

      RenderMode::TerminalAnsi => {
        if !context.previous_has_same_typename {
          writable.write_str(self.get_ansi_color(grapheme, context))?;
        }

        writable.write_str(grapheme.as_str())?;

        if !context.next_has_same_typename {
          writable.write_str("\x1b[0m")?;
        };
      }

      RenderMode::Html => {
        if !context.previous_has_same_typename {
          writable.write_str("<pre class=\"fileforge-lib-html-output ")?;
          writable.write_str(self.get_html_class_name(grapheme, context))?;
          writable.write_str("\">")?;
        }

        writable.write_str(grapheme.as_str())?;

        if !context.next_has_same_typename {
          writable.write_str("</pre>")?;
        };
      }
    }

    Ok(())
  }

  fn get_ansi_color(&self, grapheme: Grapheme, context: CellTagContext) -> &'static str;
  fn get_rgba_color(&self, grapheme: Grapheme, context: CellTagContext) -> (u8, u8, u8, u8);
  fn get_html_class_name(&self, grapheme: Grapheme, context: CellTagContext) -> &'static str;

  fn write_hover_text(
    &self,
    writable: &mut dyn Write,
    grapheme: Grapheme,
    context: CellTagContext,
  ) -> Result<(), core::fmt::Error>;

  /// Should return `core::any::type_name<Self>()`
  /// cannot do this in the trait implementation because it would
  /// always return `dyn CellTag`
  fn get_name(&self) -> &'static str;
}
