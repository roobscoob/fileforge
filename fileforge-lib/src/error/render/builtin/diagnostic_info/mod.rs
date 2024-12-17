use core::fmt::Debug;

use heapless::Vec;

use crate::{
  diagnostic::node::{
    branch::DiagnosticBranch, name::DiagnosticNodeName, reference::DiagnosticReference,
  },
  error::{
    render::{
      buffer::{
        canvas::RenderBufferCanvas,
        cell::tag::builtin::diagnostic_info::{
          DIAGNOSTIC_INFO_NAME, DIAGNOSTIC_LOCATION, DIAGNOSTIC_LOCATION_SEPARATOR,
          DIAGNOSTIC_SEPARATOR, DIAGNOSTIC_VALUE_SEPARATOR,
        },
      },
      position::RenderPosition,
      r#trait::renderable::Renderable,
    },
    report::note::{set::ReportNoteSet, ReportNote},
  },
};

mod stack_linked_list;

use self::stack_linked_list::StackSinglyLinkedList;

use super::{
  arrow::{primary::PrimaryArrow, secondary::SecondaryArrow, EitherArrow},
  number::formatted_unsigned::FormattedUnsigned,
  transformation::Transformation,
};

pub enum DiagnosticInfoTail<'l, 't, 'a, 'b, const NAME_SIZE: usize> {
  PathSeparator(&'l DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE>),
  Transformation(
    Transformation,
    &'l DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE>,
  ),
  Arrow(
    usize,
    heapless::Vec<
      (
        Option<Transformation>,
        &'l DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE>,
      ),
      0x10,
    >,
  ),
  Diagnostic(
    // determines if this should render the note
    bool,
    &'l ReportNote<'t, 'a, 'b, NAME_SIZE>,
    Option<&'l dyn Renderable<'t>>,
    heapless::Vec<
      (
        Option<Transformation>,
        &'l DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE>,
      ),
      0x10,
    >,
  ),
  None,
}

impl<'l, 't, 'a, 'b, const NAME_SIZE: usize> Debug
  for DiagnosticInfoTail<'l, 't, 'a, 'b, NAME_SIZE>
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      DiagnosticInfoTail::Arrow(..) => f.write_str("DiagnosticInfoTail::Arrow(..)"),
      DiagnosticInfoTail::Diagnostic(..) => f.write_str("DiagnosticInfoTail::Diagnostic(..)"),
      DiagnosticInfoTail::None => f.write_str("DiagnosticInfoTail::None"),
      DiagnosticInfoTail::PathSeparator(..) => f.write_str("DiagnosticInfoTail::PathSeparator(..)"),
      DiagnosticInfoTail::Transformation(..) => {
        f.write_str("DiagnosticInfoTail::Transformation(..)")
      }
    }
  }
}

#[derive(Debug)]
pub struct DiagnosticInfo<'l, 't, 'a, 'b, const NAME_SIZE: usize> {
  name: DiagnosticNodeName<NAME_SIZE>,
  offset_in_parent: u64,
  length: u64,
  tail: DiagnosticInfoTail<'l, 't, 'a, 'b, NAME_SIZE>,
  should_display_length: bool,
}

impl<'l, 't, 'a, 'b, const NAME_SIZE: usize> DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE> {
  // turns each diagnostic into a DiagnosticInfo
  fn local_transform_diagnostic<
    Cb: for<'x> FnOnce(
      Option<
        &'x StackSinglyLinkedList<(
          DiagnosticReference<'b, NAME_SIZE>,
          DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE>,
        )>,
      >,
    ),
  >(
    note_set: &'l ReportNoteSet<'t, 'a, 'b, NAME_SIZE>,
    note: &'l ReportNote<'t, 'a, 'b, NAME_SIZE>,
    index: usize,
    previous: Option<
      &StackSinglyLinkedList<(
        DiagnosticReference<'b, NAME_SIZE>,
        DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE>,
      )>,
    >,
    callback: Cb,
  ) {
    if index >= note.locations().len() {
      return callback(previous);
    }

    let c = note.locations().skip(index).next().unwrap();
    let v = c.dereference_expect("Expected valid reference when building a report");

    // if we are a parent of any other node
    // english:
    //   of all locations that are not a note location (unless they are a parent of a note location)
    //   find if any of those locations are our current location
    if note_set
      .notes
      .iter()
      .flat_map(|note| note.locations())
      .flat_map(|l| l.reference.parents())
      .any(|p| p == v)
    {
      // skip (will create later!)
      Self::local_transform_diagnostic(note_set, note, index + 1, previous, callback);
      return;
    }

    let should_render_note = note.locations.last().eq(&Some(c));

    let d = DiagnosticInfo {
      name: v.name,
      offset_in_parent: match v.branch {
        DiagnosticBranch::None => 0,
        DiagnosticBranch::Logical { .. } => 0,
        DiagnosticBranch::Physical { offset, .. } => offset,
      },
      length: v.size,
      should_display_length: match v.branch {
        DiagnosticBranch::None => false,
        DiagnosticBranch::Logical { .. } => false,
        DiagnosticBranch::Physical { offset, parent } => {
          offset + v.size
            != parent
              .relocate(c.reference.pool)
              .dereference_expect("Expected valid reference when building a report")
              .size
        }
      },
      tail: DiagnosticInfoTail::Diagnostic(should_render_note, note, c.value, Vec::new()),
    };

    let l = StackSinglyLinkedList {
      contents: (c.reference, d),
      previous,
    };

    Self::local_transform_diagnostic(note_set, note, index + 1, Some(&l), callback)
  }

  // calls local_transform_diagnostic for each diagnostic, storing the result in a stack linked list
  fn local_transform_diagnostics<
    Cb: for<'x> FnOnce(
      Option<
        &'x StackSinglyLinkedList<(
          DiagnosticReference<'b, NAME_SIZE>,
          DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE>,
        )>,
      >,
    ),
  >(
    notes: &'l ReportNoteSet<'t, 'a, 'b, NAME_SIZE>,
    index: usize,
    previous: Option<
      &StackSinglyLinkedList<(
        DiagnosticReference<'b, NAME_SIZE>,
        DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE>,
      )>,
    >,
    callback: Cb,
  ) {
    if index >= notes.notes.len() {
      return callback(previous);
    }

    let note = notes.notes.get(index).unwrap();

    Self::local_transform_diagnostic(notes, note, 0, previous, |l| {
      Self::local_transform_diagnostics(notes, index + 1, l, callback);
    });
  }

  fn local_tree_recurse<'g, Cb: for<'x> FnMut(&'x DiagnosticInfo<'x, 't, 'a, 'b, NAME_SIZE>)>(
    notes: &'l ReportNoteSet<'t, 'a, 'b, NAME_SIZE>,
    array: Option<
      &'g StackSinglyLinkedList<(
        DiagnosticReference<'b, NAME_SIZE>,
        DiagnosticInfo<'g, 't, 'a, 'b, NAME_SIZE>,
      )>,
    >,
    called_back: Option<&StackSinglyLinkedList<&DiagnosticReference<'b, NAME_SIZE>>>,
    mut callback: Cb,
  ) {
    let y = 'wider: {
      'wide: for (reference, info) in array.iter().flat_map(|v| v.iter()) {
        let parent = reference
          .dereference_expect("Expected valid reference when building a report")
          .branch
          .parent();

        if parent.is_none() {
          if called_back
            .iter()
            .flat_map(|r| r.iter())
            .find(|clb| ***clb == *reference)
            .is_none()
          {
            // 1. perform callback

            callback(info);

            // 2. push on to stack

            let n = StackSinglyLinkedList {
              contents: reference,
              previous: called_back,
            };

            Self::local_tree_recurse(notes, array, Some(&n), callback);

            return;
          } else {
            continue;
          }
        }

        let parent_reference = parent.unwrap().relocate(reference.pool);
        let parent = parent_reference
          .dereference_expect("Expected valid parent reference when building a report");

        if array.iter().flat_map(|v| v.iter()).any(|i| {
          i.0
            .dereference_expect("Expected valid reference when building a report")
            == parent
        }) {
          continue;
        }

        let mut new = DiagnosticInfo {
          name: parent.name,
          offset_in_parent: match parent.branch {
            DiagnosticBranch::None => 0,
            DiagnosticBranch::Logical { .. } => 0,
            DiagnosticBranch::Physical { offset, .. } => offset,
          },
          length: parent.size,
          should_display_length: match parent.branch {
            DiagnosticBranch::None => false,
            DiagnosticBranch::Logical { .. } => false,
            DiagnosticBranch::Physical {
              offset,
              parent: parent2,
            } => {
              offset + parent.size
                != parent2
                  .relocate(reference.pool)
                  .dereference_expect("Expected valid reference when building a report")
                  .size
            }
          },
          tail: if let Some(x) = notes
            .notes
            .iter()
            .find(|note| note.locations().any(|l| l.reference == parent_reference))
          {
            DiagnosticInfoTail::Diagnostic(
              x.locations().last().unwrap().reference == parent_reference,
              x,
              x.locations()
                .find(|l| l.reference == parent_reference)
                .unwrap()
                .value,
              Vec::new(),
            )
          } else {
            DiagnosticInfoTail::None
          },
        };

        let mut indent = 1;

        for parent_of in parent_reference.parents() {
          let mut child_count = 0;

          'counting: for (child_index, child) in notes
            .notes
            .iter()
            .flat_map(|v| v.locations())
            .flat_map(|l| l.reference.parents_incl_self())
            .filter(|l| {
              l.branch
                .parent()
                .map(|p| {
                  p.relocate(reference.pool)
                    .dereference_expect("Expected valid parent reference when building a report")
                    .eq(&parent_of)
                })
                .unwrap_or(false)
            })
            .enumerate()
          {
            'checking_dupes: for (other_child_index, other_child) in notes
              .notes
              .iter()
              .flat_map(|v| v.locations())
              .flat_map(|l| l.reference.parents_incl_self())
              .filter(|l| {
                l.branch
                  .parent()
                  .map(|p| {
                    p.relocate(reference.pool)
                      .dereference_expect("Expected valid parent reference when building a report")
                      .eq(&parent_of)
                  })
                  .unwrap_or(false)
              })
              .enumerate()
            {
              if other_child == child && other_child_index != child_index {
                continue 'counting;
              }

              if other_child == child {
                break 'checking_dupes;
              }
            }

            child_count += 1;
          }

          if child_count >= 2 {
            indent += 2;
          }
        }

        'itera: for (child_index, child) in notes
          .notes
          .iter()
          .flat_map(|v| v.locations())
          .flat_map(|l| l.reference.parents_incl_self())
          .filter(|l| {
            l.branch
              .parent()
              .map(|p| p.relocate(reference.pool).eq(&parent_reference))
              .unwrap_or(false)
          })
          .enumerate()
        {
          'checking_dupes: for (other_child_index, other_child) in notes
            .notes
            .iter()
            .flat_map(|v| v.locations())
            .flat_map(|l| l.reference.parents_incl_self())
            .filter(|l| {
              l.branch
                .parent()
                .map(|p| p.relocate(reference.pool).eq(&parent_reference))
                .unwrap_or(false)
            })
            .enumerate()
          {
            if other_child == child && other_child_index != child_index {
              continue 'itera;
            }

            if other_child == child {
              break 'checking_dupes;
            }
          }

          let child = match array {
            None => None,
            Some(i) => 'pl: {
              for v in i.iter() {
                if v
                  .0
                  .dereference_expect("Expected valid parent reference when building a report")
                  == child
                {
                  break 'pl Some(v);
                }
              }

              break 'pl None;
            }
          };

          if child.is_none() {
            continue 'wide;
          }

          let (reference, info) = child.unwrap();

          match &mut new.tail {
            DiagnosticInfoTail::Diagnostic(_, _, _, ref mut vec) => {
              match reference
                .dereference_expect("Expected valid parent reference when building a report")
                .branch
              {
                DiagnosticBranch::None => unreachable!(),
                DiagnosticBranch::Logical { name, .. } => vec
                  .push((Some(Transformation { name }), info))
                  .map_err(|_| {})
                  .unwrap(),
                DiagnosticBranch::Physical { .. } => {
                  vec.push((None, info)).map_err(|_| {}).unwrap()
                }
              };
            }

            DiagnosticInfoTail::None => {
              new.tail = match reference
                .dereference_expect("Expected valid parent reference when building a report")
                .branch
              {
                DiagnosticBranch::None => unreachable!(),
                DiagnosticBranch::Logical { name, .. } => {
                  DiagnosticInfoTail::Transformation(Transformation { name }, &info)
                }
                DiagnosticBranch::Physical { .. } => DiagnosticInfoTail::PathSeparator(&info),
              };
            }

            DiagnosticInfoTail::PathSeparator(other_ref) => {
              let mut vec = heapless::Vec::new();

              vec.push((None, *other_ref)).map_err(|_| {}).unwrap();

              match reference
                .dereference_expect("Expected valid parent reference when building a report")
                .branch
              {
                DiagnosticBranch::None => unreachable!(),
                DiagnosticBranch::Logical { name, .. } => vec
                  .push((Some(Transformation { name }), info))
                  .map_err(|_| {})
                  .unwrap(),
                DiagnosticBranch::Physical { .. } => {
                  vec.push((None, info)).map_err(|_| {}).unwrap()
                }
              };

              new.tail = DiagnosticInfoTail::Arrow(indent, vec);
            }

            DiagnosticInfoTail::Transformation(transformation, other_ref) => {
              let mut vec = heapless::Vec::new();

              vec
                .push((Some(*transformation), *other_ref))
                .map_err(|_| {})
                .unwrap();

              match reference
                .dereference_expect("Expected valid parent reference when building a report")
                .branch
              {
                DiagnosticBranch::None => unreachable!(),
                DiagnosticBranch::Logical { name, .. } => vec
                  .push((Some(Transformation { name }), info))
                  .map_err(|_| {})
                  .unwrap(),
                DiagnosticBranch::Physical { .. } => {
                  vec.push((None, info)).map_err(|_| {}).unwrap()
                }
              };

              new.tail = DiagnosticInfoTail::Arrow(indent, vec);
            }

            DiagnosticInfoTail::Arrow(_, ref mut vec) => {
              match reference
                .dereference_expect("Expected valid parent reference when building a report")
                .branch
              {
                DiagnosticBranch::None => unreachable!(),
                DiagnosticBranch::Logical { name, .. } => vec
                  .push((Some(Transformation { name }), info))
                  .map_err(|_| {})
                  .unwrap(),
                DiagnosticBranch::Physical { .. } => {
                  vec.push((None, info)).map_err(|_| {}).unwrap()
                }
              };
            }
          }
        }

        break 'wider StackSinglyLinkedList {
          contents: (parent_reference, new),
          previous: array,
        };
      }
      return;
    };

    Self::local_tree_recurse(notes, Some(&y), called_back, callback);
  }

  pub fn transform_diagnostics<Cb: for<'x> FnMut(&'x DiagnosticInfo<'x, 't, 'a, 'b, NAME_SIZE>)>(
    notes: &'l ReportNoteSet<'t, 'a, 'b, NAME_SIZE>,
    callback: Cb,
  ) {
    Self::local_transform_diagnostics(notes, 0, None, |res| {
      Self::local_tree_recurse(notes, res, None, callback);
    });
  }
}

impl<'l, 't, 'a, 'b, const NAME_SIZE: usize> Renderable<'t>
  for DiagnosticInfo<'l, 't, 'a, 'b, NAME_SIZE>
{
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.set_tagged_str(self.name.as_str(), &DIAGNOSTIC_INFO_NAME);

    let start_offset = FormattedUnsigned::new(self.offset_in_parent as u64)
      .with_base(16)
      .with_uppercase()
      .with_tag(&DIAGNOSTIC_LOCATION);

    let end_offset = FormattedUnsigned::new(self.offset_in_parent as u64 + self.length as u64)
      .with_base(16)
      .with_uppercase()
      .with_tag(&DIAGNOSTIC_LOCATION);

    let max_padding = usize::max(start_offset.length(), end_offset.length());

    let start_offset = start_offset.with_padding(max_padding);
    let end_offset = end_offset.with_padding(max_padding);

    if self.offset_in_parent != 0 || self.should_display_length {
      canvas.set_tagged_str("@[", &DIAGNOSTIC_LOCATION_SEPARATOR);

      if self.offset_in_parent != 0 {
        canvas.set_tagged_str("0×", &DIAGNOSTIC_LOCATION_SEPARATOR);

        canvas.write(&start_offset)?;
      }

      canvas.set_tagged_str("..", &DIAGNOSTIC_LOCATION_SEPARATOR);

      if self.should_display_length {
        canvas.set_tagged_str("0×", &DIAGNOSTIC_LOCATION_SEPARATOR);

        canvas.write(&end_offset)?;
      }

      canvas.set_tagged_char("]", &DIAGNOSTIC_LOCATION_SEPARATOR);
    }

    let end_position = canvas.get_position();

    match &self.tail {
      DiagnosticInfoTail::None => {}

      DiagnosticInfoTail::PathSeparator(next) => {
        canvas.cursor_right();
        canvas.set_tagged_char("/", &DIAGNOSTIC_SEPARATOR);
        canvas.cursor_right();
        canvas.write(*next)?;
      }

      DiagnosticInfoTail::Transformation(transformation, next) => {
        canvas.cursor_right();
        canvas.write(transformation)?;
        canvas.cursor_right();
        canvas.write(*next)?;
      }

      DiagnosticInfoTail::Diagnostic(should_render_note, note, value, children) => {
        let result = if let Some(value) = value {
          canvas.cursor_right();
          canvas.set_tagged_char("=", &DIAGNOSTIC_VALUE_SEPARATOR);
          canvas.cursor_right();
          Some(canvas.write(*value)?)
        } else {
          None
        };

        // draw ~~~

        let indent = canvas.get_start_position().column();

        canvas
          .set_position(canvas.get_start_position())
          .cursor_down();

        let mut current_char: &'static str = "^";

        for column in canvas.start_position.column()..end_position.column() {
          canvas.set_column(column);

          if let Some(tag) = note.tag {
            canvas.set_tagged_char(current_char, tag);
          } else {
            canvas.set_char(current_char);
          }

          current_char = "~"
        }

        if let Some(result) = result {
          if result.get_line_height() == 1 {
            if let Some(tag) = note.tag {
              canvas.set_tagged_str("~~~", tag);
            } else {
              canvas.set_str("~~~");
            }

            for column in result.start_position.column()..result.end_position.column() {
              canvas.set_column(column);

              if let Some(tag) = note.tag {
                canvas.set_tagged_char("~", tag);
              } else {
                canvas.set_char("~");
              }
            }
          }
        }

        let pos = canvas.get_position();

        let result = if *should_render_note {
          canvas
            .set_column(canvas.start_position.column())
            .cursor_down()
            .cursor_right_by(if children.len() == 0 { 0 } else { 2 });

          canvas.write(note.message)?.get_line_height()
        } else {
          0
        };

        canvas.set_position(pos);
        canvas.set_column(canvas.start_position.column());

        let mut next_arrow = SecondaryArrow {
          height: result,
          transformation: None,
          replace_last: false,
        };

        let mut next_arrow_position = canvas.get_position().down(1);

        for (transformation, element) in children.iter() {
          canvas.set_position(next_arrow_position);

          let arrow_result = canvas.write(&SecondaryArrow {
            transformation: *transformation,
            height: next_arrow.height,
            replace_last: next_arrow.replace_last,
          })?;
          let element_result = canvas.write(*element)?;

          next_arrow = SecondaryArrow {
            height: element_result.get_line_height() - 1,
            transformation: None,
            replace_last: true,
          };
          next_arrow_position = RenderPosition::new(arrow_result.end_position.line(), indent);
        }

        canvas.cursor_down_by(2);
      }

      DiagnosticInfoTail::Arrow(indent, list) => {
        let mut next_arrow = EitherArrow::Primary(PrimaryArrow {
          cradle_width: end_position.column() - canvas.start_position.column(),
          indent: *indent,
          transformation: None,
        });

        let mut next_arrow_position = canvas.start_position.down(1);

        for (transformation, element) in list.iter() {
          canvas.set_position(next_arrow_position);

          let arrow_result = canvas.write(&next_arrow.with_transformation(*transformation))?;
          let element_result = canvas.write(*element)?;

          next_arrow = EitherArrow::Secondary(SecondaryArrow {
            height: element_result.get_line_height() - 1,
            transformation: None,
            replace_last: true,
          });
          next_arrow_position = RenderPosition::new(arrow_result.end_position.line(), *indent);
        }

        canvas.cursor_down();
      }
    }

    Ok(())
  }
}
