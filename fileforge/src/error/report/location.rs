use crate::{
  diagnostic::{node::reference::DiagnosticReference, pool::DiagnosticPoolProvider},
  error::render::r#trait::renderable::Renderable,
};

pub struct ReportLocation<'t, 'l, 'pool> {
  pub(crate) reference: DiagnosticReference<'pool>,
  pub(crate) value: Option<&'l dyn Renderable<'t>>,
}

impl<'t, 'l, 'pool> ReportLocation<'t, 'l, 'pool> {
  pub fn dereference<P: DiagnosticPoolProvider>(&self, provider: &P) -> Option<P::Node> {
    self.reference.dereference(provider)
  }
}

impl<'t, 'l, 'pool> PartialEq
  for ReportLocation<'t, 'l, 'pool>
{
  fn eq(&self, other: &Self) -> bool {
    self.reference == other.reference
      && self.value.is_some() == other.value.is_some()
      && if self.value.is_some() && other.value.is_some() {
        core::ptr::eq(self.value.unwrap(), other.value.unwrap())
      } else {
        true
      }
  }
}
