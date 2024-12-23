use crate::{
  diagnostic::node::reference::DiagnosticReference, error::Error, provider::error::ProviderError,
};

pub struct UnderlyingProviderStatError<Se: ProviderError>(pub Se);

impl<Se: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for UnderlyingProviderStatError<Se>
{
  fn with_report<Cb: FnMut(crate::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(
    &self,
    callback: Cb,
  ) {
    todo!()
  }
}
