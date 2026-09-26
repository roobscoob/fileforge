use fileforge::{
  binary_reader::{
    error::{common::SeekOffset, seek_out_of_bounds::SeekOutOfBounds, SkipError},
    readable::IntoReadable,
    BinaryReader,
  },
  diagnostic::{
    pool::DiagnosticPoolProvider,
    value::{DiagnosticSaturation, DiagnosticValue},
  },
  error::{
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT},
      builtin::{number::formatted_unsigned::FormattedUnsigned, text::r#const::ConstText},
    },
    report::{note::ReportNote, Report},
    FileforgeError,
  },
  stream::ReadableStream,
};
use fileforge_macros::{story, text};

use crate::{
  report::{render_with_context, CORRUPTED, TRUNCATED, TRUNCATED_FILE_NOTE},
  sead::sarc::{
    header::{readable::SarcHeaderReadError, SarcHeader},
    names_offset,
    sfat::{
      entry::SFAT_ENTRY_SIZE,
      header::{readable::SfatHeaderReadError, SfatHeader},
      name_table::header::{SfntHeader, SfntHeaderReadError},
    },
    Sarc,
  },
};

/// Where the data offset sits in the SARC header.
const DATA_OFFSET_POSITION: u64 = 0xC;

impl<'pool, S: ReadableStream<Type = u8>> IntoReadable<'pool, S> for Sarc<'pool, S> {
  type Argument = ();
  type Error = SarcReadError<'pool, S>;

  async fn read(mut reader: BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    let header: SarcHeader = reader.read().await.map_err(SarcReadError::Header)?;
    let sfat: SfatHeader = reader.read().await.map_err(SarcReadError::SfatHeader)?;

    reader.skip(sfat.file_count as u64 * SFAT_ENTRY_SIZE).await.map_err(SarcReadError::Entries)?;

    let _: SfntHeader = reader.read().await.map_err(SarcReadError::SfntHeader)?;

    let names_offset = names_offset(sfat.file_count);

    if (header.data_offset as u64) < names_offset {
      let position = DATA_OFFSET_POSITION as i128 - reader.stream().offset() as i128;

      return Err(SarcReadError::DataInsideTables {
        data_offset: reader.create_physical_diagnostic(position, Some(4), "DataOffset").saturate(header.data_offset),
        tables_end: names_offset,
      });
    }

    Ok(Sarc { reader, header, sfat })
  }
}

#[story("file ends inside the file entries", SarcReadError::<StoryStream>::Entries(fileforge::binary_reader::error::SkipError::OutOfBounds(fileforge::binary_reader::error::seek_out_of_bounds::SeekOutOfBounds {
  seek_offset: fileforge::binary_reader::error::common::SeekOffset::InBounds(96),
  provider_size: DiagnosticValue(64, None),
  container_dr: dr!("archive.sarc" @ 0..64),
})))]
#[story("data starts inside the tables", {
  let error: SarcReadError<'_, StoryStream> = SarcReadError::DataInsideTables {
    data_offset: dv!("archive.sarc" / "DataOffset" @ 12..16 [0x10u32]),
    tables_end: 0x58,
  };
  error
})]
pub enum SarcReadError<'pool, S: ReadableStream<Type = u8>> {
  Header(SarcHeaderReadError<'pool, S::ReadError>),
  SfatHeader(SfatHeaderReadError<'pool, S::ReadError>),
  /// Skipping over the SFAT's entries, to the SFNT, failed.
  Entries(SkipError<'pool, S::SkipError>),
  SfntHeader(SfntHeaderReadError<'pool, S::ReadError>),
  /// The header says the file data starts before the tables end.
  DataInsideTables { data_offset: DiagnosticValue<'pool, u32>, tables_end: u64 },
}

const SKIPPING_ENTRIES: ConstText = ConstText::new("This happened while skipping over the SARC archive's file entries.", &REPORT_INFO_LINE_TEXT);

impl<'pool, S: ReadableStream<Type = u8>> FileforgeError for SarcReadError<'pool, S> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Header(error) => error.render_into_report(provider, callback),
      Self::SfatHeader(error) => error.render_into_report(provider, callback),
      Self::Entries(SkipError::OutOfBounds(SeekOutOfBounds {
        seek_offset: SeekOffset::InBounds(entries_end),
        provider_size,
        container_dr,
      })) => {
        let hex = |value: u64| FormattedUnsigned::new(value as u128).base(16).uppercase().prefix("0x");
        let entries_end = hex(*entries_end);
        let length = FormattedUnsigned::new(**provider_size as u128).separator(3, ",");
        let length_hex = hex(**provider_size);

        let ends_text = text!([&REPORT_ERROR_TEXT] "The file ends partway through the SARC archive's file entries, which run until {&entries_end}.");
        let length_text = text!([&REPORT_INFO_LINE_TEXT] "The file is {&length} ({&length_hex}) bytes long.");

        let mut report = Report::new::<Self>(provider, &"Truncated SARC archive").with_info_line(&ends_text).with_info_line(&length_text).with_flag_line(&TRUNCATED);

        if let Some(container) = container_dr {
          report.add_note(ReportNote::new(&TRUNCATED_FILE_NOTE).with_location(container).with_tag(&REPORT_ERROR_TEXT));
        }

        report.apply(callback);
      }
      Self::Entries(error) => render_with_context(error, &SKIPPING_ENTRIES, provider, callback),
      Self::SfntHeader(error) => error.render_into_report(provider, callback),
      Self::DataInsideTables { data_offset, tables_end } => {
        let hex = |value: u64| FormattedUnsigned::new(value as u128).base(16).uppercase().prefix("0x");
        let data_start = hex(**data_offset as u64);
        let tables_end = hex(*tables_end);

        let offset_text = text!([&REPORT_ERROR_TEXT] "The file data is said to start at {&data_start}, but the archive's tables run until {&tables_end}.");
        let note_text = text!([&REPORT_INFO_LINE_TEXT] "The data offset");
        let location = data_offset.map(|offset| hex(offset as u64));

        let mut report = Report::new::<Self>(provider, &"SARC data overlaps its tables").with_info_line(&offset_text).with_flag_line(&CORRUPTED);

        if location.reference().is_some() {
          report.add_note(ReportNote::new(&note_text).with_location(&location).with_tag(&REPORT_INFO_LINE_TEXT));
        }

        report.apply(callback);
      }
    }
  }
}
