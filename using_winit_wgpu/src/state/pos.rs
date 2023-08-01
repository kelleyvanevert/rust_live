use std::cmp::Ordering;

use super::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub row: i32,
    pub col: i32,
}

impl Pos {
    pub fn order(a: Pos, b: Pos) -> (Pos, Pos) {
        if a < b {
            (a, b)
        } else {
            (b, a)
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
