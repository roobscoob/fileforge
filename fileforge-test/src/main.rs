use fileforge_lib::{
  diagnostic::{
    node::{branch::DiagnosticBranch, name::DiagnosticNodeName},
    pool::{
      fixed::{entry::DiagnosticPoolEntry, FixedDiagnosticPool},
      DiagnosticPool,
    },
  },
  error::{
    render::{
      buffer::{
        cell::{
          tag::{builtin::report::REPORT_ERROR_TEXT, context::RenderMode},
          RenderBufferCell,
        },
        RenderBuffer,
      },
      position::RenderPosition,
    },
    report::{kind::ReportKind, note::ReportNote, Report},
  },
};
use fileforge_macros::text;

struct DemoError;

fn main() {
  let mut entries: [DiagnosticPoolEntry<32>; 100] = core::array::from_fn(|_| DiagnosticPoolEntry::default());
  let pool = FixedDiagnosticPool::new(&mut entries);

  let root = pool.create(DiagnosticBranch::None, Some(1024), DiagnosticNodeName::from("SomeFile.demo"));
  let c1 = root.create_physical_child(32, Some(64), DiagnosticNodeName::from("FileTable"));
  let c2 = c1.create_physical_child(16, Some(16), DiagnosticNodeName::from("[1] (Contents.demo)"));
  let c3 = c2.create_physical_child(0, Some(8), DiagnosticNodeName::from("Offset"));
  let c4 = c2.create_physical_child(8, Some(8), DiagnosticNodeName::from("Length"));

  let t1 = text!([&REPORT_ERROR_TEXT] "Demo Flag Line");
  let t2 = text!([&REPORT_ERROR_TEXT] "Demo Info Line");
  let t3 = text!([&REPORT_ERROR_TEXT] "Demo Note");

  let mut report: Report<'_, '_, '_, 32> = Report::new::<DemoError>(ReportKind::Error, "Reader Exhausted");

  report = report.with_flag_line(&t1).unwrap();
  report = report.with_info_line(&t2).unwrap();
  report = report.with_note(|| ReportNote::new(&t3)).unwrap();

  let mut o = 0;

  loop {
    let mut buffer = [RenderBufferCell::default(); 80];
    let mut buffer = RenderBuffer::new(&mut buffer, 80, o);
    o += 1;

    let r = buffer.canvas_at(RenderPosition::zero()).write(&report).unwrap();

    if r.get_line_height() <= o {
      break;
    }

    let mut s = String::new();

    buffer.flush_into(&mut s, RenderMode::TerminalAnsi).unwrap();

    print!("{}", s);
  }

  drop(report);
}
