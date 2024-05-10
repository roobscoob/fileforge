use core::slice::Iter;

use crate::{diagnostic::node::reference::DiagnosticReference, error::render::{buffer::cell::tag::CellTag, r#trait::renderable::Renderable}};

use super::location::ReportLocation;

pub mod set;

pub struct ReportNote<'t, 'l, 'pool_lifetime, const NODE_NAME_SIZE: usize> {
  pub (crate) locations: heapless::Vec<ReportLocation<'t, 'l, 'pool_lifetime, NODE_NAME_SIZE>, 0x10>,
  pub (crate) message: &'l dyn Renderable<'t>,
  pub (crate) tag: Option<&'t dyn CellTag>,
}

impl<'t, 'l, 'pool_lifetime, const NODE_NAME_SIZE: usize> ReportNote<'t, 'l, 'pool_lifetime, NODE_NAME_SIZE> {
  pub fn new(message: &'l dyn Renderable<'t>) -> Self {
    ReportNote { locations: Default::default(), message, tag: None }
  }

  pub fn with_location(mut self, reference: DiagnosticReference<'pool_lifetime, NODE_NAME_SIZE>, value: &'l dyn Renderable<'t>) -> Result<Self, ()> {
    self.locations.push(ReportLocation { reference, value: Some(value) }).map_err(|_| {})?;

    Ok(self)
  }

  pub fn with_unvalued_location(mut self, reference: DiagnosticReference<'pool_lifetime, NODE_NAME_SIZE>) -> Result<Self, ()> {
    self.locations.push(ReportLocation { reference, value: None }).map_err(|_| {})?;

    Ok(self)
  }

  pub fn with_raw_location(mut self, location: ReportLocation<'t, 'l, 'pool_lifetime, NODE_NAME_SIZE>) -> Result<Self, ()> {
    self.locations.push(location).map_err(|_| {})?;

    Ok(self)
  }

  pub fn with_tag(mut self, tag: &'t dyn CellTag) -> Self {
    self.tag = Some(tag);

    self
  }

  pub fn locations<'a>(&'a self) -> Iter<'a, ReportLocation<'t, 'l, 'pool_lifetime, NODE_NAME_SIZE>> {
    self.locations.iter()
  }
}

impl<'t, 'l, 'pool_lifetime, const NODE_NAME_SIZE: usize> Eq for ReportNote<'t, 'l, 'pool_lifetime, NODE_NAME_SIZE> {}
impl<'t, 'l, 'pool_lifetime, const NODE_NAME_SIZE: usize> PartialEq for ReportNote<'t, 'l, 'pool_lifetime, NODE_NAME_SIZE> {
  fn eq(&self, other: &Self) -> bool {
    core::ptr::eq(self.message, other.message) && self.locations == other.locations
  }
}