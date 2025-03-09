use fileforge_macros::text;

use crate::{diagnostic::{node::reference::DiagnosticReference, pool::DiagnosticPoolProvider, value::DiagnosticValue}, error::{context::ErrorContext, render::{buffer::cell::tag::builtin::report::{REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT}, builtin::number::formatted_unsigned::FormattedUnsigned}, report::{kind::ReportKind, location::ReportLocation, note::ReportNote, Report}, FileforgeError}};

pub struct ReaderExhaustedError<'pool, const IS_READ: bool> {
    pub container: Option<DiagnosticReference<'pool>>,
    pub length: DiagnosticValue<'pool, u64>,
    pub offset: u64,
    pub stream_length: DiagnosticValue<'pool, u64>,
}

impl<'pool> FileforgeError for ReaderExhaustedError<'pool,true> {
    fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(&self, provider: &'pool_ref P, mut callback: impl for<'a, 'b, 'p2> FnMut(crate::error::report::Report<'a, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> ()) {
        let context = ErrorContext::new(provider).with("length", self.length.reference()).with("stream_length", self.stream_length.reference()).with("container", self.container);

        let overflow_size = (self.offset as u128 + *self.length as u128) - *self.stream_length as u128;
        let overflow_size_base_10 = FormattedUnsigned::new(overflow_size).separator(3, ",");

        let container_size_base_10 = FormattedUnsigned::new(*self.stream_length as u128).separator(3, ",");
        let container_size_base_16 = FormattedUnsigned::new(*self.stream_length as u128).base(16).uppercase().prefix("0x");

        let length_base_10 = FormattedUnsigned::new(*self.length as u128).separator(3, ",");
        let length_base_16 = FormattedUnsigned::new(*self.length as u128).base(16).uppercase().prefix("0x");

        let read_offset_base_16 = FormattedUnsigned::new(self.offset as u128).base(16).uppercase().prefix("0x");
    
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
            {context.has("length")}
                [&REPORT_INFO_LINE_TEXT] "The read requested {&length_base_10} ({&length_base_16}) bytes at offset {&read_offset_base_16}. This is where the read length originated from.",

            [&REPORT_INFO_LINE_TEXT] "The read requested {&length_base_10} ({&length_base_16}) bytes at offset {&read_offset_base_16}"
        );
        
        let mut report = Report::new::<Self>(provider, ReportKind::Error, "Reader Exhausted")
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

        if let Some(location) = context.get("length") {
            report
                .add_note(ReportNote::new(&read_text).with_tag(&REPORT_INFO_LINE_TEXT).with_raw_location(ReportLocation { value: Some(&length_base_16), reference: location }).unwrap())
                .unwrap()
        } else {
            report.add_info_line(&read_text).unwrap()
        }

        callback(report);
    }
}

impl<'pool> FileforgeError for ReaderExhaustedError<'pool,false> {
    fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(&self, provider: &'pool_ref P, mut callback: impl for<'a, 'b, 'p2> FnMut(crate::error::report::Report<'a, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> ()) {
        let context = ErrorContext::new(provider).with("length", self.length.reference()).with("stream_length", self.stream_length.reference()).with("container", self.container);

        let overflow_size = (self.offset as u128 + *self.length as u128) - *self.stream_length as u128;
        let overflow_size_base_10 = FormattedUnsigned::new(overflow_size).separator(3, ",");

        let container_size_base_10 = FormattedUnsigned::new(*self.stream_length as u128).separator(3, ",");
        let container_size_base_16 = FormattedUnsigned::new(*self.stream_length as u128).base(16).uppercase().prefix("0x");

        let length_base_10 = FormattedUnsigned::new(*self.length as u128).separator(3, ",");
        let length_base_16 = FormattedUnsigned::new(*self.length as u128).base(16).uppercase().prefix("0x");

        let read_offset_base_16 = FormattedUnsigned::new(self.offset as u128).base(16).uppercase().prefix("0x");
    
        let report_text = text!(
            {overflow_size == 1}
                [&REPORT_INFO_LINE_TEXT] "Attempted to write 1 byte outside the range provided by the provider.",

            [&REPORT_INFO_LINE_TEXT] "Attempted to write {&overflow_size_base_10} bytes outside the range provided by the provider."
        );

        let provider_text = text!(
            {context.has("stream_length")}
                [&REPORT_INFO_LINE_TEXT] "Here, the provider supplied {&container_size_base_10} ({&container_size_base_16}) bytes",

            [&REPORT_INFO_LINE_TEXT] "The provider supplied {&container_size_base_10} ({&container_size_base_16}) bytes"
        );

        let read_text = text!(
            {context.has("length")}
                [&REPORT_INFO_LINE_TEXT] "The write requested {&length_base_10} ({&length_base_16}) bytes at offset {&read_offset_base_16}. This is where the write length originated from.",

            [&REPORT_INFO_LINE_TEXT] "The write requested {&length_base_10} ({&length_base_16}) bytes at offset {&read_offset_base_16}"
        );
        
        let mut report = Report::new::<Self>(provider, ReportKind::Error, "Reader Exhausted")
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

        if let Some(location) = context.get("length") {
            report
                .add_note(ReportNote::new(&read_text).with_tag(&REPORT_INFO_LINE_TEXT).with_raw_location(ReportLocation { value: Some(&length_base_16), reference: location }).unwrap())
                .unwrap()
        } else {
            report.add_info_line(&read_text).unwrap()
        }

        callback(report);
    }
}