use super::reference::DislocatedDiagnosticReference;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum DiagnosticBranch {
  #[default]
  None,

  Physical { parent: DislocatedDiagnosticReference, offset: u64 },

  Logical { parent: DislocatedDiagnosticReference, name: &'static str },
}

impl DiagnosticBranch {
  pub fn parent(&self) -> Option<DislocatedDiagnosticReference> {
    match self {
      DiagnosticBranch::None => None,
      DiagnosticBranch::Physical { parent, .. } => Some(*parent),
      DiagnosticBranch::Logical { parent, .. } => Some(*parent)
    }
  }
}