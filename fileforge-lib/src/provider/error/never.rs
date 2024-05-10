use core::fmt::Debug;

use super::ProviderError;

pub struct Never;

impl Debug for Never {
  fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    unreachable!()
  }
}

impl ProviderError for Never {
  fn with_report_given_location<'pool_lifetime, Cb: FnMut(crate::error::report::Report<NODE_NAME_SIZE>) -> (), const NODE_NAME_SIZE: usize>(&self, _: crate::diagnostic::node::reference::DiagnosticReference<'pool_lifetime, NODE_NAME_SIZE>, _: Cb) {
    unreachable!()
  }
}