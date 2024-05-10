use crate::{diagnostic::node::{reference::DiagnosticReference, DiagnosticNode}, error::render::r#trait::renderable::Renderable};

pub struct ReportLocation<'t, 'l, 'pool_lifetime, const NODE_NAME_SIZE: usize> {
  pub (crate) reference: DiagnosticReference<'pool_lifetime, NODE_NAME_SIZE>,
  pub (crate) value: Option<&'l dyn Renderable<'t>>,
}

impl<'t, 'l, 'pool_lifetime, const NODE_NAME_SIZE: usize> ReportLocation<'t, 'l, 'pool_lifetime, NODE_NAME_SIZE> {
  pub fn dereference(&self) -> Option<DiagnosticNode<NODE_NAME_SIZE>> {
    self.reference.dereference()
  }

  pub fn dereference_expect(&self, message: &str) -> DiagnosticNode<NODE_NAME_SIZE> {
    self.reference.dereference_expect(message)
  }
}

impl<'t, 'l, 'pool_lifetime, const NODE_NAME_SIZE: usize> PartialEq for ReportLocation<'t, 'l, 'pool_lifetime, NODE_NAME_SIZE> {
  fn eq(&self, other: &Self) -> bool {
    self.reference == other.reference && self.value.is_some() == other.value.is_some() && if self.value.is_some() && other.value.is_some() { core::ptr::eq(self.value.unwrap(), other.value.unwrap()) } else { true }
  }
}