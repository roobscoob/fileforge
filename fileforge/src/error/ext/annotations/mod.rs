use crate::{diagnostic::pool::DiagnosticPoolProvider, error::report::Report};

pub mod annotated;

pub trait Annotation {
  fn attach<'t1, 'b1, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: P,
    current_report: Report<'t1, 'b1, ITEM_NAME_SIZE, P>,
    callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  );
}
