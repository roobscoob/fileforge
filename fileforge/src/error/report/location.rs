use crate::{
  diagnostic::{
    node::reference::{DiagnosticReference, DislocatedDiagnosticReference},
    pool::DiagnosticPoolProvider,
    value::{DiagnosticValue, DislocatedDiagnosticValue},
  },
  error::render::r#trait::renderable::Renderable,
};

#[derive(Clone, Copy)]
pub struct ReportLocation<'t, 'l> {
  pub(crate) reference: DislocatedDiagnosticReference,
  pub(crate) value: Option<&'l dyn Renderable<'t>>,
}

impl<'t, 'l> ReportLocation<'t, 'l> {
  pub fn dereference<P: DiagnosticPoolProvider>(&self, provider: &P) -> Option<P::Node> {
    self.reference.relocate(provider.get_builder()).dereference(provider)
  }
}

impl<'t, 'l> PartialEq for ReportLocation<'t, 'l> {
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

impl<'l, 't, T: Renderable<'t>> TryFrom<&'l DislocatedDiagnosticValue<T>> for ReportLocation<'t, 'l> {
  type Error = ();

  fn try_from(value: &'l DislocatedDiagnosticValue<T>) -> Result<Self, Self::Error> {
    if let Some(reference) = value.reference() {
      Ok(ReportLocation {
        reference,
        value: Some(value.value_ref()),
      })
    } else {
      Err(())
    }
  }
}

impl<'l, 't, 'pool, T: Renderable<'t>> TryFrom<&'l DiagnosticValue<'pool, T>> for ReportLocation<'t, 'l> {
  type Error = ();

  fn try_from(value: &'l DiagnosticValue<T>) -> Result<Self, Self::Error> {
    if let Some(reference) = value.reference() {
      Ok(ReportLocation {
        reference: reference.dislocate(),
        value: Some(value.value()),
      })
    } else {
      Err(())
    }
  }
}

impl<'l, 't, 'a> TryFrom<&'a DislocatedDiagnosticReference> for ReportLocation<'t, 'l> {
  type Error = ();

  fn try_from(value: &'a DislocatedDiagnosticReference) -> Result<Self, Self::Error> {
    Ok(ReportLocation { reference: *value, value: None })
  }
}

impl<'l, 't> TryFrom<DislocatedDiagnosticReference> for ReportLocation<'t, 'l> {
  type Error = ();

  fn try_from(value: DislocatedDiagnosticReference) -> Result<Self, Self::Error> {
    Ok(ReportLocation { reference: value, value: None })
  }
}

impl<'l, 't, 'pool, 'a> TryFrom<&'a DiagnosticReference<'pool>> for ReportLocation<'t, 'l> {
  type Error = ();

  fn try_from(value: &'a DiagnosticReference<'pool>) -> Result<Self, Self::Error> {
    Ok(ReportLocation {
      reference: value.dislocate(),
      value: None,
    })
  }
}

impl<'l, 't, 'pool> TryFrom<DiagnosticReference<'pool>> for ReportLocation<'t, 'l> {
  type Error = ();

  fn try_from(value: DiagnosticReference<'pool>) -> Result<Self, Self::Error> {
    Ok(ReportLocation {
      reference: value.dislocate(),
      value: None,
    })
  }
}

impl<'l, 't, 'a> TryFrom<&'a ReportLocation<'t, 'l>> for ReportLocation<'t, 'l> {
  type Error = ();

  fn try_from(value: &'a ReportLocation<'t, 'l>) -> Result<Self, Self::Error> {
    Ok(*value)
  }
}
