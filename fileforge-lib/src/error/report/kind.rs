#[derive(Clone, Copy, Debug, Default)]
pub enum ReportKind {
  #[default]
  Error,
  Warning,
}