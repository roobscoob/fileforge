use core::fmt::Debug;

use crate::error::Error;

use super::ProviderError;

pub struct Never;

impl Debug for Never {
  fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { unreachable!() }
}

impl ProviderError for Never {
  fn with_report<
    'pool,
    Cb: FnMut(crate::error::report::Report<NODE_NAME_SIZE>) -> (),
    const NODE_NAME_SIZE: usize,
  >(
    &self,
    location: Option<
      crate::diagnostic::node::reference::DiagnosticReference<'pool, NODE_NAME_SIZE>,
    >,
    callback: Cb,
  ) {
    unreachable!()
  }
}

impl<const NODE_NAME_SIZE: usize> Error<NODE_NAME_SIZE> for Never {
  fn with_report<Cb: FnMut(crate::error::report::Report<NODE_NAME_SIZE>) -> ()>(&self, _: Cb) {
    unreachable!()
  }
}
