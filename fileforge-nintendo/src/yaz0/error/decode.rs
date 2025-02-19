use fileforge_lib::{diagnostic::{node::{name::DiagnosticNodeName, reference::DiagnosticReference}, value::DiagnosticValue}, error::FileforgeError, reader::{error::get_primitive::GetPrimitiveError, readable::error::user::UserReadableError, Reader}, stream::{error::user_read::UserReadError, ReadableStream}};

pub enum Yaz0DecodeError<'pool, const NODE_NAME_SIZE: usize, StreamReadError: UserReadError<NODE_NAME_SIZE>> {
    MissingHeader {
        group_header_diagnostic: Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>,
        get_primitive_error: GetPrimitiveError<'pool, NODE_NAME_SIZE, StreamReadError>,
    },
    MissingInlineByte {
        chunk_index: u8,
        group_header: DiagnosticValue<'pool, u8, NODE_NAME_SIZE>,
        chunk_diagnostic: Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>,
        get_primitive_error: GetPrimitiveError<'pool, NODE_NAME_SIZE, StreamReadError>,
    },
    MissingBackreference {
        chunk_index: u8,
        group_header: DiagnosticValue<'pool, u8, NODE_NAME_SIZE>,
        chunk_diagnostic: Option<DiagnosticReference<'pool, NODE_NAME_SIZE>>,
        get_primitive_error: GetPrimitiveError<'pool, NODE_NAME_SIZE, StreamReadError>,
    },
}

impl<'pool, const NODE_NAME_SIZE: usize, StreamReadError: UserReadError<NODE_NAME_SIZE>> Yaz0DecodeError<'pool, NODE_NAME_SIZE, StreamReadError> {
    pub fn missing_header<S: ReadableStream<NODE_NAME_SIZE>>(
        error: GetPrimitiveError<'pool, NODE_NAME_SIZE, StreamReadError>,
        reader: &Reader<'pool, NODE_NAME_SIZE, S>,
        group_index: u32,
    ) -> Yaz0DecodeError<'pool, NODE_NAME_SIZE, StreamReadError> {
        let group_diagnostic = reader.create_physical_diagnostic(0, None, DiagnosticNodeName::from_named_index("Group", group_index as u64));
        let group_header_diagnostic = group_diagnostic.as_ref().map(|v| v.create_physical_child(0, Some(1), "Header"));

        Yaz0DecodeError::MissingHeader { group_header_diagnostic, get_primitive_error: error }
    }

    pub fn missing_inline_byte<S: ReadableStream<NODE_NAME_SIZE>>(
        error: GetPrimitiveError<'pool, NODE_NAME_SIZE, StreamReadError>,
        reader: &Reader<'pool, NODE_NAME_SIZE, S>,
        chunk_index: u8,
        group_header: u8,
        byte_offset: u8,
        group_index: u32,
    ) -> Yaz0DecodeError<'pool, NODE_NAME_SIZE, StreamReadError> {
        let mut length = Some(0);

        for chunk_num in chunk_index..8 {
            let mask = 1u8 << chunk_num;
            let is_fixed = (group_header & mask) != 0;

            if !is_fixed {
                length = None;
            } else {
                length = length.map(|l| l + 1);
            }
        }

        let group_diagnostic = reader.create_physical_diagnostic(-(byte_offset as i64), length.map(|l| (l + byte_offset) as u64), DiagnosticNodeName::from_named_index("Group", group_index as u64));
        let group_header_diagnostic = group_diagnostic.as_ref().map(|v| v.create_physical_child(0, Some(1), "Header"));
        let chunk_diagnostic = group_diagnostic.as_ref().map(|v| v.create_physical_child(byte_offset as u64, Some(1), DiagnosticNodeName::from_named_index("Chunk", chunk_index as u64)));

        Yaz0DecodeError::MissingInlineByte {
            group_header: DiagnosticValue(group_header, group_header_diagnostic),
            get_primitive_error: error,
            chunk_diagnostic,
            chunk_index,
        }
    }

    pub fn missing_assumed_short_backreference<S: ReadableStream<NODE_NAME_SIZE>>(
        error: GetPrimitiveError<'pool, NODE_NAME_SIZE, StreamReadError>,
        reader: &Reader<'pool, NODE_NAME_SIZE, S>,
        chunk_index: u8,
        group_header: u8,
        byte_offset: u8,
        group_index: u32,
    ) -> Yaz0DecodeError<'pool, NODE_NAME_SIZE, StreamReadError> {
        let group_diagnostic = reader.create_physical_diagnostic(-(byte_offset as i64), None, DiagnosticNodeName::from_named_index("Group", group_index as u64));
        let group_header_diagnostic = group_diagnostic.as_ref().map(|v| v.create_physical_child(0, Some(1), "Header"));
        let chunk_diagnostic = group_diagnostic.as_ref().map(|v| v.create_physical_child(byte_offset as u64, Some(1), DiagnosticNodeName::from_named_index("Chunk", chunk_index as u64)));

        Yaz0DecodeError::MissingBackreference {
            group_header: DiagnosticValue(group_header, group_header_diagnostic),
            get_primitive_error: error,
            chunk_diagnostic,
            chunk_index,
        }
    }

    pub fn missing_long_backreference<S: ReadableStream<NODE_NAME_SIZE>>(
        error: GetPrimitiveError<'pool, NODE_NAME_SIZE, StreamReadError>,
        reader: &Reader<'pool, NODE_NAME_SIZE, S>,
        chunk_index: u8,
        group_header: u8,
        byte_offset: u8,
        group_index: u32,
    ) -> Yaz0DecodeError<'pool, NODE_NAME_SIZE, StreamReadError> {
        let group_diagnostic = reader.create_physical_diagnostic(-(byte_offset as i64), None, DiagnosticNodeName::from_named_index("Group", group_index as u64));
        let group_header_diagnostic = group_diagnostic.as_ref().map(|v| v.create_physical_child(0, Some(1), "Header"));
        let chunk_diagnostic = group_diagnostic.as_ref().map(|v| v.create_physical_child(byte_offset as u64, Some(1), DiagnosticNodeName::from_named_index("Chunk", chunk_index as u64)));

        Yaz0DecodeError::MissingBackreference {
            group_header: DiagnosticValue(group_header, group_header_diagnostic),
            get_primitive_error: error,
            chunk_diagnostic,
            chunk_index,
        }
    }
}

impl<'pool, const NODE_NAME_SIZE: usize, StreamReadError: UserReadError<NODE_NAME_SIZE>> UserReadableError<'pool, NODE_NAME_SIZE> for Yaz0DecodeError<'pool, NODE_NAME_SIZE, StreamReadError> {}

impl<'pool, const NODE_NAME_SIZE: usize, StreamReadError: UserReadError<NODE_NAME_SIZE>> FileforgeError<'pool, NODE_NAME_SIZE> for Yaz0DecodeError<'pool, NODE_NAME_SIZE, StreamReadError> {
    fn render_into_report(&self, callback: impl for<'a, 'b> FnMut(fileforge_lib::error::report::Report<'a, 'b, 'pool, NODE_NAME_SIZE>) -> ()) {
        unimplemented!()
    }
}

