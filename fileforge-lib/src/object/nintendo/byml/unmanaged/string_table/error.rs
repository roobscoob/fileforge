use crate::{provider::{r#trait::Provider, error::ProviderError}, reader::{error::{out_of_bounds::ReadOutOfBoundsError, underlying_provider_read::UnderlyingProviderReadError, ExpectationFailedError}, r#trait::primitive::Primitive}, error::render::{builtin::text::Text, buffer::cell::tag::CellTag, r#trait::renderable::Renderable}, diagnostic::node::reference::DiagnosticReference};

pub enum StringTableSizeError<'pool, Pe: ProviderError, ErrorFn: for <'primitive, 'closure> Fn(&'primitive u8, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  UnderlyingProviderReadError(UnderlyingProviderReadError<'pool, Pe, DIAGNOSTIC_NODE_NAME_SIZE>),
  
  MissingNodeKind(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  InvalidNodeKind(ExpectationFailedError<'pool, ErrorFn, u8, 1, DIAGNOSTIC_NODE_NAME_SIZE>),

  MissingElementCount(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),

  NotLargeEnough {
    expectation_source_reference: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
    expectation_source_value: u64,
    expectation_value: u64,
    actual_value: u64,
  },
}

impl<'pool, Pe: ProviderError, ErrorFn: for <'primitive, 'closure> Fn(&'primitive u8, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), const DIAGNOSTIC_NODE_NAME_SIZE: usize> From<UnderlyingProviderReadError<'pool, Pe, DIAGNOSTIC_NODE_NAME_SIZE>> for StringTableSizeError<'pool, Pe, ErrorFn, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn from(value: UnderlyingProviderReadError<'pool, Pe, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::UnderlyingProviderReadError(value)
  }
}