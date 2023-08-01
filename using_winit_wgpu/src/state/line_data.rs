use super::{direction::Direction, pos::Pos, selection::Selection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Char(char),
    Widget { id: usize, width: usize },
}

impl Cell {
    pub fn width(&self) -> usize {
        match self {
            Cell::Char(_) => 1,
            Cell::Widget { width, .. } => *width,
        }
    }
}

pub struct LineData(Vec<Vec<Cell>>);

impl LineData {
    pub fn new() -> LineData {
        LineData(vec![vec![]])
    }

    pub fn from_str(code: &str) -> LineData {
        LineData(
            code.split('\n')
                .map(|line| line.chars().map(|ch| Cell::Char(ch)).collect::<Vec<_>>())
                .collect(),
        )
    }

    pub fn with_widget_at_pos(mut self, pos: Pos, id: usize, width: usize) -> Self {
        self.insert(pos, vec![Cell::Widget { id, width }]);
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn line_width(&self, row: i32) -> i32 {
        if row < 0 || row >= self.0.len() as i32 {
            return 0;
        }

        self.0[row as usize].iter().map(Cell::width).sum::<usize>() as i32
    }

    pub fn lines(&self) -> &Vec<Vec<Cell>> {
        &self.0
    }

    // invariant: caret is at snapped position
    pub fn move_selection_caret(&self, selection: &mut Selection, dir: Direction, selecting: bool) {
        let mut caret = selection.caret;

        debug_assert_eq!(caret, self.snap(caret).0);

        let prev_col = caret.col;

        match dir {
            Direction::Up => {
                if caret.row <= 0 {
                    caret.col = 0;
                } else {
                    caret.row -= 1;
                    caret = self
                        .snap(caret.with_col(selection.desired_col.unwrap_or(caret.col)))
                        .0;
                }
            }
            Direction::Down => {
                if caret.row >= self.len() as i32 - 1 {
                    caret.col = self.line_width(self.len() as i32 - 1);
                } else {
                    caret.row += 1;
                    caret = self
                        .snap(caret.with_col(selection.desired_col.unwrap_or(caret.col)))
                        .0;
                }
            }
            Direction::Right => {
                if caret.col == self.line_width(caret.row) {
                    if caret.row < self.len() as i32 - 1 {
                        caret.row += 1;
                        caret.col = 0;
                    }
                } else {
                    let cell = self
                        .snap_nearest(caret)
                        .3
                        .expect("cannot move caret right, because no cell exists at position");
                    caret.col += cell.width() as i32;
                }
            }
            Direction::Left => {
                if caret.col == 0 {
                    if caret.row > 0 {
                        caret.row -= 1;
                        caret.col = self.line_width(caret.row);
                    }
                } else {
                    let cell = self
                        .snap_nearest(caret)
                        .2
                        .expect("cannot move caret left, because no cell exists at position");
                    caret.col -= cell.width() as i32;
                }
            }
        }

        selection.move_caret_to(caret, selecting);

        if dir == Direction::Up || dir == Direction::Down {
            selection.desired_col = selection.desired_col.or(Some(prev_col));
        } else {
            selection.desired_col = None;
        }
    }

    pub fn insert(&mut self, pos: Pos, cells: Vec<Cell>) -> usize {
        let (Pos { row, .. }, i) = self.snap(pos);

        let inserted_len = cells.iter().map(|cell| cell.width()).sum::<usize>();

        self.0[row as usize].splice(i..i, cells);

        inserted_len
    }

    /**
     Snaps the given pos to the neasest valid caret position, and returns:

     - the snapped/nearest valid caret position
     - the index in that row
     - the cell at the previous position (which can be `None` if at the start of a line);
     - the cell at that position (which can be `None` if at the end of a line);
     - whether the position was inside the text (end of line is valid);
     - whether the position was valid (i.e. not inside a widget).
    */
    pub fn snap_nearest(&self, pos: Pos) -> (Pos, usize, Option<Cell>, Option<Cell>, bool, bool) {
        let empty_line = &vec![];

        let mut inside = true;
        let mut valid = true;

        // snap to a valid row + its line
        let (row, line) = if pos.row < 0 {
            inside = false;
            valid = false;
            (0, self.0.get(0).unwrap_or(empty_line))
        } else if let Some(line) = self.0.get(pos.row as usize) {
            (pos.row, line)
        } else {
            inside = false;
            valid = false;
            (self.0.len() as i32 - 1, self.0.last().unwrap_or(empty_line))
        };

        let line_width = self.line_width(row);

        // snap to a valid cell
        let (col, i, prev_cell, cell) = if pos.col < 0 {
            inside = false;
            valid = false;
            (0, 0, None, None)
        } else if pos.col <= line_width {
            let mut i = 0;
            let mut col = 0;
            let mut prev_cell = None;
            loop {
                let cell = line.get(i).map(|&c| c);
                match cell {
                    None => break (pos.col, i, prev_cell, cell),
                    _ if col == pos.col => break (pos.col, i, prev_cell, cell),
                    Some(cell) => {
                        // edge-case: if clicking within a widget,
                        //  but closer to the end than the start,
                        //  then select the column after
                        let col_next = col + (cell.width() as i32);
                        if col_next > pos.col {
                            // (if we're at a widget, then this can happen)
                            valid = false;
                            if col_next - pos.col >= pos.col - col {
                                break (col, i, prev_cell, Some(cell));
                            } else {
                                break (col_next, i + 1, Some(cell), line.get(i + 1).map(|&c| c));
                            }
                        }

                        i += 1;
                        col = col_next;
                        prev_cell = Some(cell);
                    }
                }
            }
        } else {
            inside = false;
            valid = false;
            (line_width, self.0[row as usize].len(), None, None)
        };

        let snapped = Pos { row, col };

        (snapped, i, prev_cell, cell, inside, valid)
    }

    /** Snaps the pos to the nearest available position */
    pub fn snap(&self, pos: Pos) -> (Pos, usize) {
        let (pos, i, _, _, _, _) = self.snap_nearest(pos);
        (pos, i)
    }
}

// impl std::ops::Index<Pos> for LineData {
//     type Output = Option<Cell>;

//     fn index(&self, pos: Pos) -> &Self::Output {
//         &self.snap_nearest(pos).2
//     }
// }

// impl std::ops::IndexMut<Pos> for LineData {
//     fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
//         todo!()
//     }
// }
