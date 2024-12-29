use report::Report;

pub mod context;
pub mod render;
pub mod report;

pub trait FileforgeError<const NODE_NAME_SIZE: usize> {
  fn render_into_report(&self, callback: impl FnMut(Report<NODE_NAME_SIZE>) -> ());
}

impl<const NODE_NAME_SIZE: usize> FileforgeError<NODE_NAME_SIZE> for core::convert::Infallible {
  fn render_into_report(&self, _: impl FnMut(Report<NODE_NAME_SIZE>) -> ()) { unreachable!() }
}
