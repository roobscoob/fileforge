use core::any::type_name;

use fileforge_macros::text;

use crate::{
  binary_reader::error::common::ExhaustedType,
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    ext::annotations::Annotation,
    render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::r#const::ConstText},
    report::Report,
  },
};

pub struct PrimitiveName<T: ExhaustedType> {
  primitive_name: ConstText,
  t: T,
}

impl<T: ExhaustedType> PrimitiveName<T> {
  pub fn for_type<T2>() -> Self {
    Self {
      primitive_name: ConstText::new_untagged(type_name::<T2>()),
      t: T::VALUE,
    }
  }
}

impl<T: ExhaustedType> Annotation for PrimitiveName<T> {
  fn attach<'t1, 'b1, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    _: P,
    current_report: Report<'t1, 'b1, ITEM_NAME_SIZE, P>,
    callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    let message = self.t.message(&REPORT_INFO_LINE_TEXT);
    let t = text!([&REPORT_INFO_LINE_TEXT] "This error originated while attempting to {&message} {&self.primitive_name}");

    current_report.with_info_line(&t).apply(callback)
  }
}
