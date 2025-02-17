use core::marker::PhantomData;

use crate::{
  diagnostic::node::{reference::DiagnosticReference, DiagnosticNode},
  error::render::buffer::cell::tag::builtin::report::REPORT_FLAG_LINE_TEXT,
};

use super::report::Report;

pub trait ErrorContextNode<'pool, const NODE_NAME_SIZE: usize> {
  fn try_get(&self, key: &str) -> Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>;
  fn any_missing(&self) -> bool;
}

pub struct NoneContextNode;

impl<'pool, const NODE_NAME_SIZE: usize> ErrorContextNode<'pool, NODE_NAME_SIZE> for NoneContextNode {
  fn try_get(&self, _: &str) -> Option<DiagnosticReference<'pool, NODE_NAME_SIZE>> { None }
  fn any_missing(&self) -> bool { false }
}

pub struct SomeContextNode<'pool, T: ErrorContextNode<'pool, NODE_NAME_SIZE>, const NODE_NAME_SIZE: usize> {
  left: (&'static str, Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>),
  right: T,
}

impl<'pool, T: ErrorContextNode<'pool, NODE_NAME_SIZE>, const NODE_NAME_SIZE: usize> ErrorContextNode<'pool, NODE_NAME_SIZE> for SomeContextNode<'pool, T, NODE_NAME_SIZE> {
  fn try_get(&self, key: &str) -> Option<DiagnosticReference<'pool, NODE_NAME_SIZE>> {
    if self.left.1.is_some() && self.left.1.unwrap().family_exists() && self.left.0 == key {
      return Some(self.left.1.unwrap());
    }

    self.right.try_get(key)
  }

  fn any_missing(&self) -> bool { self.left.1.is_none() || self.right.any_missing() }
}

pub struct ErrorContext<'pool, T: ErrorContextNode<'pool, NODE_NAME_SIZE>, const NODE_NAME_SIZE: usize> {
  pd: PhantomData<&'pool ()>,
  data: T,
}

impl<'pool, const NODE_NAME_SIZE: usize> ErrorContext<'pool, NoneContextNode, NODE_NAME_SIZE> {
  pub fn new() -> ErrorContext<'pool, NoneContextNode, NODE_NAME_SIZE> {
    ErrorContext {
      pd: PhantomData,
      data: NoneContextNode,
    }
  }
}

impl<'pool, T: ErrorContextNode<'pool, NODE_NAME_SIZE>, const NODE_NAME_SIZE: usize> ErrorContext<'pool, T, NODE_NAME_SIZE> {
  pub fn with(self, key: &'static str, value: Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>) -> ErrorContext<'pool, SomeContextNode<'pool, T, NODE_NAME_SIZE>, NODE_NAME_SIZE> {
    ErrorContext {
      pd: PhantomData,
      data: SomeContextNode {
        left: (key, value.and_then(|value| value.dereference().map(|_| value))),
        right: self.data,
      },
    }
  }

  pub fn any_missing(&self) -> bool { self.data.any_missing() }

  pub fn get(&self, key: &str) -> Option<DiagnosticReference<'pool, NODE_NAME_SIZE>> { self.data.try_get(key) }
}

impl<'l, 'pool, const NODE_NAME_SIZE: usize> Report<'static, 'l, 'pool, NODE_NAME_SIZE> {
  pub fn with_error_context<N: ErrorContextNode<'pool, NODE_NAME_SIZE>>(self, context: &ErrorContext<'pool, N, NODE_NAME_SIZE>) -> Self {
    if context.any_missing() {
      self
        .with_flag_line(const_text!([&REPORT_FLAG_LINE_TEXT] "Some Diagnostics failed to load. You are seeing a minified version with what available data exists."))
        .unwrap()
    } else {
      self
    }
  }
}
