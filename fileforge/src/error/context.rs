use crate::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    render::{buffer::cell::tag::builtin::report::REPORT_FLAG_LINE_TEXT, r#trait::renderable::Renderable},
    report::{location::ReportLocation, note::ReportNote},
  },
};

use super::report::Report;

pub trait ErrorContextNode<'t, 'l> {
  fn try_get<P: DiagnosticPoolProvider + Clone>(&self, key: &str, provider: P) -> Option<ReportLocation<'t, 'l>>;
  fn any_missing(&self) -> bool;
}

pub struct NoneContextNode;

impl<'t, 'l> ErrorContextNode<'t, 'l> for NoneContextNode {
  fn try_get<P: DiagnosticPoolProvider + Clone>(&self, _: &str, _: P) -> Option<ReportLocation<'t, 'l>> {
    None
  }
  fn any_missing(&self) -> bool {
    false
  }
}

pub struct SomeContextNode<'t, 'l, T: ErrorContextNode<'t, 'l>> {
  left: (&'static str, Option<ReportLocation<'t, 'l>>),
  right: T,
}

impl<'t, 'l, T: ErrorContextNode<'t, 'l>> ErrorContextNode<'t, 'l> for SomeContextNode<'t, 'l, T> {
  fn try_get<P: DiagnosticPoolProvider + Clone>(&self, key: &str, provider: P) -> Option<ReportLocation<'t, 'l>> {
    if self.left.0 == key {
      if let Some(v) = self.left.1 {
        if v.reference.relocate(provider.get_builder()).family_exists(&provider) {
          return Some(v);
        }
      }
    }

    self.right.try_get(key, provider)
  }

  fn any_missing(&self) -> bool {
    self.left.1.is_none() || self.right.any_missing()
  }
}

pub struct ErrorContext<T, P: DiagnosticPoolProvider + Clone> {
  provider: P,
  data: T,
}

impl<P: DiagnosticPoolProvider + Clone> ErrorContext<NoneContextNode, P> {
  pub fn new(provider: P) -> ErrorContext<NoneContextNode, P> {
    ErrorContext { provider, data: NoneContextNode }
  }
}

impl<'t: 'l, 'l, P: DiagnosticPoolProvider + Clone, T: ErrorContextNode<'t, 'l>> ErrorContext<T, P> {
  pub fn with_some<'pool>(self, key: &'static str, value: impl TryInto<ReportLocation<'t, 'l>>) -> ErrorContext<SomeContextNode<'t, 'l, T>, P> {
    ErrorContext {
      provider: self.provider,
      data: SomeContextNode {
        left: (key, value.try_into().ok()),
        right: self.data,
      },
    }
  }

  pub fn with<'pool>(self, key: &'static str, value: Option<impl TryInto<ReportLocation<'t, 'l>>>) -> ErrorContext<SomeContextNode<'t, 'l, T>, P> {
    ErrorContext {
      provider: self.provider,
      data: SomeContextNode {
        left: (key, value.and_then(|v| v.try_into().ok())),
        right: self.data,
      },
    }
  }

  pub fn any_missing(&self) -> bool {
    self.data.any_missing()
  }

  pub fn get(&self, key: &str) -> Option<ReportLocation<'t, 'l>> {
    self.data.try_get(key, &self.provider)
  }

  pub fn has(&self, key: &str) -> bool {
    self.data.try_get(key, &self.provider).is_some()
  }
}

impl<'t, 'l, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider + Clone> Report<'t, 'l, ITEM_NAME_SIZE, P> {
  pub fn with_error_context(self) -> ContextReport<'t, 'l, NoneContextNode, P, ITEM_NAME_SIZE> {
    ContextReport {
      context: ErrorContext::new(self.pool().clone()),
      report: self,
    }
  }
}

pub struct ContextReport<'t, 'l, N: ErrorContextNode<'t, 'l>, P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize> {
  context: ErrorContext<N, P>,
  report: Report<'t, 'l, ITEM_NAME_SIZE, P>,
}

impl<'pool, 't, 'l, N: ErrorContextNode<'t, 'l>, P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize> ContextReport<'t, 'l, N, P, ITEM_NAME_SIZE> {
  pub fn with_context(self, key: &'static str, value: impl TryInto<ReportLocation<'t, 'l>>) -> ContextReport<'t, 'l, SomeContextNode<'t, 'l, N>, P, ITEM_NAME_SIZE> {
    ContextReport {
      context: ErrorContext {
        provider: self.context.provider,
        data: SomeContextNode {
          left: (key, value.try_into().ok()),
          right: self.context.data,
        },
      },
      report: self.report,
    }
  }
  pub fn with_opt_context(self, key: &'static str, value: Option<impl TryInto<ReportLocation<'t, 'l>>>) -> ContextReport<'t, 'l, SomeContextNode<'t, 'l, N>, P, ITEM_NAME_SIZE> {
    ContextReport {
      context: ErrorContext {
        provider: self.context.provider,
        data: SomeContextNode {
          left: (key, value.and_then(|v| v.try_into().ok())),
          right: self.context.data,
        },
      },
      report: self.report,
    }
  }

  pub fn with_contextual_note_or_info(mut self, context: &'static str, renderable: &'l dyn Renderable<'t>, modifier: impl FnOnce(ReportNote<'t, 'l>) -> ReportNote<'t, 'l>) -> Self {
    if let Some(location) = self.context.get(context) {
      self.report.add_note(modifier(ReportNote::new(renderable).with_location(location)));
    } else {
      self.report.add_info_line(renderable);
    }

    self
  }

  pub fn with_contextual_note(mut self, context: &'static str, renderable: &'l dyn Renderable<'t>, modifier: impl FnOnce(ReportNote<'t, 'l>) -> ReportNote<'t, 'l>) -> Self {
    if let Some(location) = self.context.get(context) {
      self.report.add_note(modifier(ReportNote::new(renderable).with_location(location)));
    }

    self
  }

  pub fn with_contextual_note_if(mut self, condition: bool, context: &'static str, renderable: &'l dyn Renderable<'t>, modifier: impl FnOnce(ReportNote<'t, 'l>) -> ReportNote<'t, 'l>) -> Self {
    if condition {
      if let Some(location) = self.context.get(context) {
        self.report.add_note(modifier(ReportNote::new(renderable).with_location(location)));
      }
    }

    self
  }

  pub fn finalize_context(self) -> Report<'t, 'l, ITEM_NAME_SIZE, P> {
    if self.context.any_missing() {
      self
        .report
        .with_flag_line(const_text!([&REPORT_FLAG_LINE_TEXT] "Some Diagnostics failed to load. You are seeing a minified version with what available data exists."))
    } else {
      self.report
    }
  }
}
