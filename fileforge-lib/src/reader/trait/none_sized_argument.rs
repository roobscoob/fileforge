pub trait NoneSizedArgument {
  fn from_none() -> Self;
}

impl NoneSizedArgument for () {
  fn from_none() -> Self { () }
}

impl<T> NoneSizedArgument for [T; 0] {
  fn from_none() -> Self { [] }
}
