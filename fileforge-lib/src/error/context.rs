use core::marker::PhantomData;

use crate::{
  diagnostic::{node::reference::DiagnosticReference, pool::DiagnosticPoolProvider},
  error::render::buffer::cell::tag::builtin::report::REPORT_FLAG_LINE_TEXT,
};

use super::report::Report;

pub trait ErrorContextNode<'pool> {
  fn try_get<P: DiagnosticPoolProvider>(&self, key: &str, provider: &P) -> Option<DiagnosticReference<'pool>>;
  fn any_missing(&self) -> bool;
}

pub struct NoneContextNode;

impl<'pool> ErrorContextNode<'pool> for NoneContextNode {
  fn try_get<P: DiagnosticPoolProvider>(&self, _: &str, _: &P) -> Option<DiagnosticReference<'pool>> { None }
  fn any_missing(&self) -> bool { false }
}

pub struct SomeContextNode<'pool, T: ErrorContextNode<'pool>> {
  left: (&'static str, Option<DiagnosticReference<'pool>>),
  right: T,
}

impl<'pool, T: ErrorContextNode<'pool>> ErrorContextNode<'pool> for SomeContextNode<'pool, T> {
  fn try_get<P: DiagnosticPoolProvider>(&self, key: &str, provider: &P) -> Option<DiagnosticReference<'pool>> {
    if self.left.1.is_some() && self.left.1.unwrap().family_exists(provider) && self.left.0 == key {
      return Some(self.left.1.unwrap());
    }

    self.right.try_get(key, provider)
  }

  fn any_missing(&self) -> bool { self.left.1.is_none() || self.right.any_missing() }
}

pub struct ErrorContext<'pool, 'pool_ref, T: ErrorContextNode<'pool>, P: DiagnosticPoolProvider> {
  pd: PhantomData<&'pool ()>,
  provider: &'pool_ref P,
  data: T,
}

impl<'pool, 'pool_ref, P: DiagnosticPoolProvider> ErrorContext<'pool, 'pool_ref, NoneContextNode, P> {
  pub fn new(provider: &'pool_ref P) -> ErrorContext<'pool, 'pool_ref, NoneContextNode, P> {
    ErrorContext {
      pd: PhantomData,
      provider,
      data: NoneContextNode,
    }
  }
}

impl<'pool, 'pool_ref, P: DiagnosticPoolProvider, T: ErrorContextNode<'pool>> ErrorContext<'pool, 'pool_ref, T, P> {
  pub fn with(self, key: &'static str, value: Option<DiagnosticReference<'pool>>) -> ErrorContext<'pool, 'pool_ref, SomeContextNode<'pool, T>, P> {
    ErrorContext {
      pd: PhantomData,
      provider: self.provider,
      data: SomeContextNode {
        left: (key, value.and_then(|value| value.dereference(self.provider).map(|_| value))),
        right: self.data,
      },
    }
  }

  pub fn any_missing(&self) -> bool { self.data.any_missing() }

  pub fn get(&self, key: &str) -> Option<DiagnosticReference<'pool>> { self.data.try_get(key, self.provider) }

  pub fn has(&self, key: &str) -> bool { self.data.try_get(key, self.provider).is_some() }
}

impl<'l, 'pool, 'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider> Report<'static, 'l, 'pool, 'pool_ref, ITEM_NAME_SIZE, P> {
  pub fn with_error_context<N: ErrorContextNode<'pool>>(self, context: &ErrorContext<'pool, 'pool_ref, N, P>) -> Self {
    if context.any_missing() {
      self
        .with_flag_line(const_text!([&REPORT_FLAG_LINE_TEXT] "Some Diagnostics failed to load. You are seeing a minified version with what available data exists."))
        .unwrap()
    } else {
      self
    }
  }
}
