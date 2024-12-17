pub struct DomainError<T>(pub T);

impl<T> From<T> for DomainError<T> {
  fn from(value: T) -> Self { Self(value) }
}
