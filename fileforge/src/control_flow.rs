pub trait ControlFlow {
  fn should_continue(&self) -> bool;
}

pub struct Continue<T>(pub T);

impl<T> ControlFlow for Continue<T> {
  fn should_continue(&self) -> bool {
    true
  }
}

impl<T> ControlFlow for Option<T> {
  fn should_continue(&self) -> bool {
    self.is_some()
  }
}

impl<T, E> ControlFlow for Result<T, E> {
  fn should_continue(&self) -> bool {
    self.is_ok()
  }
}

impl ControlFlow for () {
  fn should_continue(&self) -> bool {
    true
  }
}

impl<T, const COUNT: usize> ControlFlow for [T; COUNT] {
  fn should_continue(&self) -> bool {
    true
  }
}

impl<T> ControlFlow for [T] {
  fn should_continue(&self) -> bool {
    true
  }
}

impl<T: ControlFlow> ControlFlow for &T {
  fn should_continue(&self) -> bool {
    (**self).should_continue()
  }
}

impl<T: ControlFlow> ControlFlow for &mut T {
  fn should_continue(&self) -> bool {
    (**self).should_continue()
  }
}
