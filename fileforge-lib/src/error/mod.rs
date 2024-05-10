use self::report::Report;

pub mod report;
pub mod render;

pub trait Error<const NODE_NAME_SIZE: usize> {
  fn with_report<Cb: FnMut(Report<NODE_NAME_SIZE>) -> ()>(&self, callback: Cb);
}