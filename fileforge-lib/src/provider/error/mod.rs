use crate::{diagnostic::node::reference::DiagnosticReference, error::report::Report};

pub mod always;
pub mod never;
pub mod read_error;
pub mod write_error;

pub trait ProviderError {
  fn with_report_given_location<'pool, Cb: FnMut(Report<NODE_NAME_SIZE>) -> (), const NODE_NAME_SIZE: usize>(&self, location: DiagnosticReference<'pool, NODE_NAME_SIZE>, callback: Cb);
}