use crate::{provider::error::ProviderError, diagnostic::node::reference::DiagnosticReference, error::Error};

pub struct UnderlyingProviderReadError<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>(pub Re, pub DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>);

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(crate::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    todo!()
  }
}