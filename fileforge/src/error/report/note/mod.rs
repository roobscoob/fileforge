use core::slice::Iter;

use crate::{
  diagnostic::{
    node::reference::{DiagnosticReference, DislocatedDiagnosticReference},
    value::{DiagnosticValue, DislocatedDiagnosticValue},
  },
  error::render::{buffer::cell::tag::CellTag, r#trait::renderable::Renderable},
};

use super::location::ReportLocation;

pub mod set;

pub struct ReportNote<'t, 'l> {
  pub(crate) locations: heapless::Vec<ReportLocation<'t, 'l>, 0x10>,
  pub(crate) message: &'l dyn Renderable<'t>,
  pub(crate) tag: Option<&'t dyn CellTag>,
}

impl<'t, 'l> ReportNote<'t, 'l> {
  pub fn new(message: &'l dyn Renderable<'t>) -> Self {
    ReportNote {
      locations: Default::default(),
      message,
      tag: None,
    }
  }

  pub fn with_location<T: TryInto<ReportLocation<'t, 'l>>>(mut self, value: T) -> Self {
    if let Ok(v) = value.try_into() {
      self.locations.push(v).map_err(|_| {}).expect("Location Container Full");
    }

    self
  }

  pub fn with_tag(mut self, tag: &'t dyn CellTag) -> Self {
    self.tag = Some(tag);
    self
  }

  pub fn locations<'a>(&'a self) -> Iter<'a, ReportLocation<'t, 'l>> {
    self.locations.iter()
  }
}

impl<'t, 'l> Eq for ReportNote<'t, 'l> {}
impl<'t, 'l> PartialEq for ReportNote<'t, 'l> {
  fn eq(&self, other: &Self) -> bool {
    core::ptr::eq(self.message, other.message) && self.locations == other.locations
  }
}
