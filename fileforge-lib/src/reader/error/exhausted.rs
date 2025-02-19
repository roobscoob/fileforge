use fileforge_macros::text;

use crate::{diagnostic::{node::reference::DiagnosticReference, value::DiagnosticValue}, error::{context::ErrorContext, render::{buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT}, builtin::number::formatted_unsigned::FormattedUnsigned}, report::{kind::ReportKind, location::ReportLocation, note::ReportNote, Report}, FileforgeError}, stream::error::stream_exhausted::StreamExhaustedError};

pub struct ReaderExhaustedError<'pool, const NODE_NAME_SIZE: usize> {
    pub container: Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>,
    pub read_length: DiagnosticValue<'pool, u64, NODE_NAME_SIZE>,
    pub read_offset: u64,
    pub stream_length: DiagnosticValue<'pool, u64, NODE_NAME_SIZE>,
}

impl<'pool, const NODE_NAME_SIZE: usize> FileforgeError<'pool, NODE_NAME_SIZE> for ReaderExhaustedError<'pool, NODE_NAME_SIZE> {
    fn render_into_report(&self, mut callback: impl for<'a, 'b> FnMut(crate::error::report::Report<'a, 'b, 'pool, NODE_NAME_SIZE>) -> ()) {
        let context = ErrorContext::new().with("read_length", self.read_length.reference()).with("stream_length", self.stream_length.reference()).with("container", self.container);

        let overflow_size = (self.read_offset as u128 + *self.read_length as u128) - *self.stream_length as u128;
        let overflow_size_base_10 = FormattedUnsigned::new(overflow_size).separator(3, ",");

        let container_size_base_10 = FormattedUnsigned::new(*self.stream_length as u128).separator(3, ",");
        let container_size_base_16 = FormattedUnsigned::new(*self.stream_length as u128).base(16).uppercase().prefix("0x");

        let read_length_base_10 = FormattedUnsigned::new(*self.read_length as u128).separator(3, ",");
        let read_length_base_16 = FormattedUnsigned::new(*self.read_length as u128).base(16).uppercase().prefix("0x");

        let read_offset_base_16 = FormattedUnsigned::new(self.read_offset as u128).base(16).uppercase().prefix("0x");
    
        let report_text = text!(
            {overflow_size == 1}
                [&REPORT_INFO_LINE_TEXT] "Attempted to read 1 byte outside the range provided by the provider.",

            [&REPORT_INFO_LINE_TEXT] "Attempted to read {&overflow_size_base_10} bytes outside the range provided by the provider."
        );

        let provider_text = text!(
            {context.has("stream_length")}
                [&REPORT_INFO_LINE_TEXT] "Here, the provider supplied {&container_size_base_10} ({&container_size_base_16}) bytes",

            [&REPORT_INFO_LINE_TEXT] "The provider supplied {&container_size_base_10} ({&container_size_base_16}) bytes"
        );

        let read_text = text!(
            {context.has("read_length")}
                [&REPORT_INFO_LINE_TEXT] "The read requested {&read_length_base_10} ({&read_length_base_16}) bytes at offset {&read_offset_base_16}. This is where the read length originated from.",

            [&REPORT_INFO_LINE_TEXT] "The read requested {&read_length_base_10} ({&read_length_base_16}) bytes at offset {&read_offset_base_16}"
        );
        
        let mut report = Report::new::<Self>(ReportKind::Error, "Reader Exhausted")
            .with_error_context(&context)
            .with_flag_line(const_text!([&REPORT_FLAG_LINE_TEXT] "This is a low-level error, intended to be consumed by higher-level error handling code. This error is not intended to be displayed to the user. If you're seeing this error and *not* a library author, it may be confusing. Please report this error to the library author."))
            .unwrap();

        report.add_info_line(&report_text).unwrap();

        if let Some(location) = context.get("container") {
            report
                .add_note(ReportNote::new(const_text!([&REPORT_INFO_LINE_TEXT] "This is the container")).with_tag(&REPORT_INFO_LINE_TEXT).with_unvalued_location(location).unwrap())
                .unwrap();
        }

        if let Some(location) = context.get("stream_length") {
            report
                .add_note(ReportNote::new(&provider_text).with_tag(&REPORT_INFO_LINE_TEXT).with_raw_location(ReportLocation { value: Some(&container_size_base_16), reference: location }).unwrap())
                .unwrap()
        } else {
            report.add_info_line(&provider_text).unwrap()
        }

        if let Some(location) = context.get("read_length") {
            report
                .add_note(ReportNote::new(&read_text).with_tag(&REPORT_INFO_LINE_TEXT).with_raw_location(ReportLocation { value: Some(&read_length_base_16), reference: location }).unwrap())
                .unwrap()
        } else {
            report.add_info_line(&read_text).unwrap()
        }

        callback(report);
    }
} 