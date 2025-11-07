use core::future::Future;

use crate::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::{ext::annotations::Annotation, report::Report, FileforgeError},
};

pub struct Annotated<A: Annotation, T: FileforgeError> {
  annotation: A,
  error: T,
}

pub trait AnnotationExt<S, E: FileforgeError> {
  fn annotate<T: Annotation>(self, annotation: T) -> Result<S, Annotated<T, E>>;
  fn annotate_with<T: Annotation>(self, generator: impl for<'a> FnOnce(&'a E) -> T) -> Result<S, Annotated<T, E>>;
  fn annotate_with_async<T: Annotation>(self, generator: impl for<'a> AsyncFnOnce(&'a E) -> T) -> impl Future<Output = Result<S, Annotated<T, E>>>;
}

impl<S, E: FileforgeError> AnnotationExt<S, E> for Result<S, E> {
  fn annotate<T: Annotation>(self, annotation: T) -> Result<S, Annotated<T, E>> {
    self.map_err(|error| Annotated { annotation, error })
  }

  fn annotate_with<T: Annotation>(self, generator: impl for<'a> FnOnce(&'a E) -> T) -> Result<S, Annotated<T, E>> {
    self.map_err(|error| Annotated { annotation: generator(&error), error })
  }

  async fn annotate_with_async<T: Annotation>(self, generator: impl for<'a> AsyncFnOnce(&'a E) -> T) -> Result<S, Annotated<T, E>> {
    match self {
      Ok(v) => Ok(v),
      Err(error) => Err(Annotated {
        annotation: generator(&error).await,
        error,
      }),
    }
  }
}

impl<A: Annotation, T: FileforgeError> FileforgeError for Annotated<A, T> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    self.error.render_into_report(provider.clone(), |report: Report<'_, '_, ITEM_NAME_SIZE, _>| {
      self.annotation.attach(provider.clone(), report, callback);
    });
  }
}
