use core::slice::Iter;

use crate::{
  diagnostic::{node::reference::DiagnosticReference, value::DiagnosticValue},
  error::render::{buffer::cell::tag::CellTag, r#trait::renderable::Renderable},
};

use super::location::ReportLocation;

pub mod set;

pub struct ReportNote<'t, 'l, 'pool> {
  pub(crate) locations: heapless::Vec<ReportLocation<'t, 'l, 'pool>, 0x10>,
  pub(crate) message: &'l dyn Renderable<'t>,
  pub(crate) tag: Option<&'t dyn CellTag>,
}

impl<'t, 'l, 'pool> ReportNote<'t, 'l, 'pool> {
  pub fn new(message: &'l dyn Renderable<'t>) -> Self {
    ReportNote {
      locations: Default::default(),
      message,
      tag: None,
    }
  }

  pub fn with_location<'x, T: Renderable<'t>>(mut self, value: &'l DiagnosticValue<'pool, T>) -> Result<Self, ()> {
    if let Some(reference) = value.reference() {
      self
        .locations
        .push(ReportLocation {
          reference,
          value: Some(value.value_ref()),
        })
        .map_err(|_| {})?;
    }

    Ok(self)
  }

  pub fn with_unvalued_location(mut self, reference: DiagnosticReference<'pool>) -> Result<Self, ()> {
    self.locations.push(ReportLocation { reference, value: None }).map_err(|_| {})?;

    Ok(self)
  }

  pub fn with_raw_location(mut self, location: ReportLocation<'t, 'l, 'pool>) -> Result<Self, ()> {
    self.locations.push(location).map_err(|_| {})?;

    Ok(self)
  }

  pub fn with_tag(mut self, tag: &'t dyn CellTag) -> Self {
    self.tag = Some(tag);

    self
  }

  pub fn locations<'a>(&'a self) -> Iter<'a, ReportLocation<'t, 'l, 'pool>> { self.locations.iter() }
}

impl<'t, 'l, 'pool> Eq for ReportNote<'t, 'l, 'pool> {}
impl<'t, 'l, 'pool> PartialEq for ReportNote<'t, 'l, 'pool> {
  fn eq(&self, other: &Self) -> bool { core::ptr::eq(self.message, other.message) && self.locations == other.locations }
}
