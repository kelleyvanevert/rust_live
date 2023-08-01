use std::cmp::Ordering;

use super::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub row: i32,
    pub col: i32,
}

impl Pos {
    pub fn order(a: Pos, b: Pos) -> Range {
        if a < b {
            (a, b).into()
        } else {
            (b, a).into()
        }
    }

    pub fn within(self, a: Pos, b: Pos) -> bool {
        a <= self && self <= b
    }

    pub fn with_row(self, row: i32) -> Pos {
        Self { row, col: self.col }
    }

    pub fn with_col(self, col: i32) -> Pos {
        Self { row: self.row, col }
    }
}

impl Into<Pos> for Direction {
    fn into(self) -> Pos {
        match self {
            Direction::Up => (0, -1).into(),
            Direction::Right => (1, 0).into(),
            Direction::Down => (0, 1).into(),
            Direction::Left => (-1, 0).into(),
        }
    }
}

impl Into<Pos> for (i32, i32) {
    fn into(self) -> Pos {
        Pos {
            row: self.1,
            col: self.0,
        }
    }
}

impl std::ops::Add for Pos {
    type Output = Pos;

    fn add(self, other: Self) -> Self::Output {
        Pos {
            row: self.row + other.row,
            col: self.col + other.col,
        }
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.row.cmp(&other.row) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.col.cmp(&other.col),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Pos,
    pub end: Pos,
}

impl Range {
    pub fn contains(self, pos: Pos) -> bool {
        self.start <= pos && pos <= self.end
    }

    pub fn overlap(a: Range, b: Range) -> bool {
        a.contains(b.start) || a.contains(b.end) || b.contains(a.start) || b.contains(a.end)
    }

    pub fn cover(a: Range, b: Range) -> Range {
        Range {
            start: a.start.min(b.start),
            end: a.end.max(b.end),
        }
    }
}

impl From<(Pos, Pos)> for Range {
    fn from((start, end): (Pos, Pos)) -> Self {
        Self { start, end }
    }
}

#[test]
fn test_contains() {
    assert_eq!(
        Range::overlap(
            Range {
                start: Pos { row: 4, col: 8 },
                end: Pos { row: 4, col: 11 }
            },
            Range {
                start: Pos { row: 4, col: 18 },
                end: Pos { row: 4, col: 18 }
            },
        ),
        false
    );
}
