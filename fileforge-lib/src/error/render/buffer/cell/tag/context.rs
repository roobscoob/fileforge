#[derive(Clone, Copy)]
pub struct CellTagContext {
  pub mode: RenderMode,
  pub previous_has_same_typename: bool,
  pub next_has_same_typename: bool,
}

#[derive(Clone, Copy)]
pub enum RenderMode {
  PlainText,
  TerminalAnsi,
  Html,
}