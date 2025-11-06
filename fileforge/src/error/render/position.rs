#[derive(Debug, Clone, Copy)]
pub struct RenderPosition {
  line: usize,
  column: usize,
}

impl RenderPosition {
  pub fn new(line: usize, column: usize) -> Self { Self { line, column } }

  pub fn line_start(line: usize) -> Self { Self::new(line, 0) }

  pub fn zero() -> Self { Self::new(0, 0) }

  pub fn try_left(&self, count: usize) -> Option<Self> {
    if self.column < count {
      None
    } else {
      Some(Self::new(self.line, self.column - count))
    }
  }

  pub fn right(&self, count: usize) -> Self { Self::new(self.line, self.column + count) }

  pub fn try_up(&self, count: usize) -> Option<Self> {
    if self.line < count {
      None
    } else {
      Some(Self::new(self.line - count, self.column))
    }
  }

  pub fn down(&self, count: usize) -> Self { Self::new(self.line + count, self.column) }

  pub fn to<'a>(&'a self, other: &Self) -> impl Iterator<Item = RenderPosition> + 'a {
    MagicIter::new(*self, *other)
  }

  pub fn line(&self) -> usize { self.line }
  pub fn column(&self) -> usize { self.column }
}

#[derive(Debug, Clone)]
enum Direction {
  Still,
  Up,
  Down,
  Left,
  Right,
}

impl Direction {
  fn new(x: isize, y: isize) -> Direction {
    let use_y_axis = x == 0;

    if x == 0 && y == 0 {
      Direction::Still
    } else if use_y_axis {
      if y > 0 {
        Direction::Down
      } else {
        Direction::Up
      }
    } else {
      if x > 0 {
        Direction::Right
      } else {
        Direction::Left
      }
    }
  }

  fn as_vector(&self) -> (isize, isize) {
    match self {
      Direction::Still => (0, 0),
      Direction::Up => (0, -1),
      Direction::Down => (0, 1),
      Direction::Left => (-1, 0),
      Direction::Right => (1, 0),
    }
  }

  pub fn calculate_direction(start: RenderPosition, end: RenderPosition) -> Direction {
    let x_diff = end.column as isize - start.column as isize;
    let y_diff = end.line as isize - start.line as isize;

    Direction::new(x_diff, y_diff)
  }

  pub fn apply_to_render_position(&self, position: RenderPosition) -> RenderPosition {
    let (x_offset, y_offset) = self.as_vector();
    RenderPosition {
      column: position.column.saturating_add_signed(x_offset),
      line: position.line.saturating_add_signed(y_offset),
    }
  }
}

struct MagicIter {
  pub start: RenderPosition,
  pub end: RenderPosition,
  pub direction: Direction,
  pub ended: bool,
}

impl MagicIter {
  pub fn new(start: RenderPosition, end: RenderPosition) -> MagicIter {
    MagicIter {
      start,
      end,
      direction: Direction::calculate_direction(start, end),
      ended: false,
    }
  }

  fn on_axis(&self) -> bool {
    self.start.column == self.end.column || self.start.line == self.end.line
  }
}

impl Iterator for MagicIter {
  type Item = RenderPosition;

  fn next(&mut self) -> Option<Self::Item> {
    if self.ended {
      return None;
    }

    let current = self.start;

    self.start = match &self.direction {
      Direction::Still => {
        self.ended = true;
        current
      }
      dir => dir.apply_to_render_position(self.start),
    };

    if self.on_axis() {
      self.direction = Direction::calculate_direction(self.start, self.end);
    }

    Some(current)
  }
}
