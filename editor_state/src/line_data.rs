use super::{direction::Direction, pos::Pos, selection::Selection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Char(char),
    Widget { id: usize, width: usize },
}

impl Token {
    pub fn width(&self) -> usize {
        match self {
            Token::Char(_) => 1,
            Token::Widget { width, .. } => *width,
        }
    }
}

/**
    Information about a line data insertion, that can be used for moving selections afterwards.

    - The `delta` should be applied to all carets on the same line and _after_ (or at) the `start` position (where the insertion was done)
    - The `added_lines` (which is the same as `delta.row`) should be applied as a row-delta to all subsequent lines
*/
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InsertionInfo {
    pub start: Pos,
    pub end: Pos,
    pub delta: Pos,
    pub added_lines: i32,
}

/**
    Information about a line data removal, that can be used for moving selections afterwards.

    - The `delta` should be applied to all carets on the same line and _after_ (or at) the `end` position
    - The `removed_lines` (which is the same as `- delta.row`) should be applied as a (negative) row-delta to all subsequent lines
*/
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RemovalInfo {
    pub start: Pos,
    pub end: Pos,
    pub delta: Pos,
    pub removed_lines: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EditResult {
    Insertion { info: InsertionInfo },
    Removal { info: RemovalInfo },
}

pub struct LineData(Vec<Vec<Token>>);

impl LineData {
    pub fn new() -> LineData {
        LineData(vec![vec![]])
    }

    pub fn with_widget_at_pos(mut self, pos: Pos, id: usize, width: usize) -> Self {
        self.insert(pos, Token::Widget { id, width }.into());
        self
    }

    pub fn with_inserted(mut self, pos: Pos, data: LineData) -> Self {
        self.insert(pos, data);
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn line_width(&self, row: i32) -> i32 {
        if row < 0 || row >= self.0.len() as i32 {
            return 0;
        }

        self.0[row as usize].iter().map(Token::width).sum::<usize>() as i32
    }

    pub fn line_empty(&self, row: usize) -> bool {
        if row >= self.len() {
            return true;
        }

        self.0[row as usize].len() == 0
    }

    pub fn row_indentation(&self, row: usize) -> usize {
        if row >= self.len() {
            return 0;
        }

        self.0[row as usize]
            .iter()
            .take_while(|&&c| c == Token::Char(' '))
            .count()
    }

    pub fn lines(&self) -> &Vec<Vec<Token>> {
        &self.0
    }

    pub fn end(&self) -> Pos {
        let row = self.len().saturating_sub(1) as i32;

        Pos {
            row,
            col: self.line_width(row),
        }
    }

    // invariant: caret is at snapped position
    pub fn calculate_caret_move(
        &self,
        mut caret: Pos,
        desired_col: Option<i32>,
        dir: Direction,
    ) -> (Pos, Option<i32>) {
        debug_assert_eq!(caret, self.snap(caret));

        let prev_col = caret.col;

        match dir {
            Direction::Up => {
                if caret.row <= 0 {
                    caret.col = 0;
                } else {
                    caret.row -= 1;
                    caret = self.snap(caret.with_col(desired_col.unwrap_or(caret.col)));
                }
            }
            Direction::Down => {
                if caret.row >= self.len() as i32 - 1 {
                    caret.col = self.line_width(self.len() as i32 - 1);
                } else {
                    caret.row += 1;
                    caret = self.snap(caret.with_col(desired_col.unwrap_or(caret.col)));
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

        (
            caret,
            if dir == Direction::Up || dir == Direction::Down {
                desired_col.or(Some(prev_col))
            } else {
                None
            },
        )
    }

    // invariant: caret is at snapped position
    pub fn move_selection_caret(&self, selection: &mut Selection, dir: Direction, selecting: bool) {
        let (caret, desired_col) =
            self.calculate_caret_move(selection.caret, selection.desired_col, dir);

        selection.move_caret_to(caret, selecting);
        selection.desired_col = desired_col;
    }

    pub fn insert(&mut self, pos: Pos, data: LineData) -> InsertionInfo {
        debug_assert_eq!(pos, self.snap(pos));

        let data = data.0;

        let i = self.get_index_in_row(pos);
        let row = pos.row as usize;

        let mut dcol = 0;

        if let Some(first_line) = data.first() {
            let multiline = data.len() > 1;

            let range = if multiline {
                i..self.0[row].len()
            } else {
                i..i
            };

            let split_off = self.0[row].splice(range, first_line.clone());

            if !multiline {
                dcol = first_line.iter().map(|cell| cell.width()).sum::<usize>() as i32;
            } else {
                let split_off: Vec<_> = split_off.collect();

                self.0
                    .splice((row + 1)..(row + 1), data[1..].iter().cloned());

                let last_len = data.last().unwrap().len();

                let last_width = data
                    .last()
                    .unwrap()
                    .iter()
                    .map(|cell| cell.width())
                    .sum::<usize>() as i32;

                dcol = last_width - pos.col;

                self.0[row + data.len() - 1].splice(last_len..last_len, split_off);
            }
        }

        let drow = (data.len() as i32 - 1).max(0);

        let delta = Pos {
            col: dcol,
            row: drow,
        };

        InsertionInfo {
            start: pos,
            end: pos + delta,
            delta,
            added_lines: drow,
        }
    }

    pub fn remove(&mut self, start: Pos, end: Pos) -> RemovalInfo {
        debug_assert_eq!(start, self.snap(start));
        debug_assert_eq!(end, self.snap(end));
        debug_assert!(start <= end);

        let i = self.get_index_in_row(start);
        let j = self.get_index_in_row(end);

        if start.row == end.row {
            self.0[start.row as usize].splice(i..j, []);
        } else {
            let split_off: Vec<_> = self.0[end.row as usize].splice(j.., []).collect();
            self.0[start.row as usize].splice(i.., split_off);
            self.0
                .splice((start.row as usize + 1)..(end.row as usize + 1), []);
        }

        let removed_lines = end.row - start.row;

        RemovalInfo {
            start,
            end,
            delta: Pos {
                row: -removed_lines,
                col: start.col - end.col,
            },
            removed_lines,
        }
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
    pub fn snap_nearest(&self, pos: Pos) -> (Pos, usize, Option<Token>, Option<Token>, bool, bool) {
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
    pub fn snap(&self, pos: Pos) -> Pos {
        self.snap_nearest(pos).0
    }

    fn get_index_in_row(&self, pos: Pos) -> usize {
        self.snap_nearest(pos).1
    }
}

impl From<&str> for LineData {
    fn from(str: &str) -> Self {
        LineData(
            str.split('\n')
                .map(|line| line.chars().map(|ch| Token::Char(ch)).collect::<Vec<_>>())
                .collect(),
        )
    }
}

impl From<Vec<Vec<Token>>> for LineData {
    fn from(lines: Vec<Vec<Token>>) -> Self {
        LineData(lines)
    }
}

impl From<Vec<Token>> for LineData {
    fn from(line: Vec<Token>) -> Self {
        LineData(vec![line])
    }
}

impl From<Vec<char>> for LineData {
    fn from(chars: Vec<char>) -> Self {
        LineData(vec![chars.iter().map(|&ch| Token::Char(ch)).collect()])
    }
}

impl From<Token> for LineData {
    fn from(cell: Token) -> Self {
        LineData(vec![vec![cell]])
    }
}

impl From<char> for LineData {
    fn from(ch: char) -> Self {
        if ch == '\n' {
            LineData(vec![vec![], vec![]])
        } else {
            LineData(vec![vec![Token::Char(ch)]])
        }
    }
}
