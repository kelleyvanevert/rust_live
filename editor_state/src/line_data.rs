use debug_unreachable::debug_unreachable;

use crate::{Direction, Pos, Range, Selection};

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

    pub fn is_widget(&self) -> bool {
        match self {
            Token::Widget { .. } => true,
            Token::Char(_) => false,
        }
    }

    pub fn is_whitespace(&self) -> bool {
        match self {
            Token::Widget { .. } => false,
            Token::Char(ch) => *ch == ' ',
        }
    }

    pub fn is_part_of_word(&self) -> bool {
        match self {
            Token::Widget { .. } => false,
            Token::Char(ch) => ch.is_alphanumeric() || *ch == '_',
        }
    }

    pub fn is_punct(&self) -> bool {
        !self.is_part_of_word() && !self.is_whitespace() && !self.is_widget()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveVariant {
    ByToken,
    ByWord,
    UntilEnd,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn line_index_col(&self, row: i32, i: usize) -> i32 {
        if row < 0 || row >= self.0.len() as i32 {
            return 0;
        }

        self.0[row as usize][..i]
            .iter()
            .map(Token::width)
            .sum::<usize>() as i32
    }

    pub fn empty(&self) -> bool {
        self.len() == 0 || (self.len() == 1 && self.0[0].len() == 0)
    }

    pub fn line_empty(&self, row: usize) -> bool {
        if row >= self.len() {
            return true;
        }

        self.0[row as usize].len() == 0
    }

    pub fn line_indent(&self, row: usize) -> usize {
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

    pub fn joined(datas: Vec<LineData>) -> LineData {
        LineData(datas.into_iter().map(|d| d.0).flatten().collect())
    }

    // invariant: caret is at snapped position
    pub fn calculate_caret_move(
        &self,
        mut caret: Pos,
        desired_col: Option<i32>,
        dir: Direction,
        variant: MoveVariant,
    ) -> (Pos, Option<i32>) {
        debug_assert_eq!(caret, self.snap(caret));

        // used to set `desired_col` if moving vertically,
        //  after calculating new caret position
        let prev_col = caret.col;

        match (variant, dir) {
            (MoveVariant::UntilEnd, Direction::Up) => {
                caret = (0, 0).into();
            }
            (MoveVariant::UntilEnd, Direction::Down) => {
                caret = self.end();
            }
            (MoveVariant::UntilEnd, Direction::Right) => {
                caret.col = self.line_width(caret.row);
            }
            (MoveVariant::UntilEnd, Direction::Left) => {
                let indent = self.line_indent(caret.row as usize) as i32;
                if caret.col == indent {
                    caret.col = 0;
                } else {
                    caret.col = indent;
                }
            }

            (MoveVariant::ByWord, Direction::Left | Direction::Right) => 'done: {
                let delta = if dir == Direction::Left { -1 } else { 1 };

                let mut i = self.snap_indices(caret).1 as i32;
                let get = |row: i32, i: i32| {
                    if dir == Direction::Left {
                        if i == 0 {
                            None
                        } else {
                            self.0[row as usize].get(i as usize - 1)
                        }
                    } else {
                        self.0[row as usize].get(i as usize)
                    }
                };

                // possibly skip 1 leading newline
                if get(caret.row, i) == None {
                    if dir == Direction::Left {
                        if caret.row == 0 {
                            caret.col = 0;
                            break 'done;
                        }

                        caret.row += delta;
                        caret.col = self.line_width(caret.row);
                        i = self.0[caret.row as usize].len() as i32;
                    } else {
                        if caret.row >= self.len() as i32 - 1 {
                            caret.col = self.line_width(self.len() as i32 - 1);
                            break 'done;
                        }

                        caret.row += delta;
                        caret.col = 0;
                        i = 0;
                    }
                }

                // skip all leading (non-newline) whitespace
                while let Some(t) = get(caret.row, i) && t.is_whitespace() {
                    caret.col += t.width() as i32 * delta;
                    i += delta;
                }

                match get(caret.row, i) {
                    None => {
                        // if at start or end of line -> we're done
                    }
                    Some(t) if t.is_widget() => {
                        // skip over single widget
                        caret.col += t.width() as i32 * delta;
                    }
                    Some(t) if t.is_part_of_word() => {
                        // skip over entire word
                        while let Some(t) = get(caret.row, i) && t.is_part_of_word() {
                            caret.col += t.width() as i32 * delta;
                            i += delta;
                        }
                    }
                    Some(t) if t.is_punct() => {
                        // if we're at a punctuation mark, skip over the next word or widget, or a sequence of punctuation marks, but stop at whitespace

                        caret.col += t.width() as i32 * delta;
                        i += delta;

                        match get(caret.row, i) {
                            Some(t) if t.is_widget() => {
                                caret.col += t.width() as i32 * delta;
                            }
                            Some(t) if t.is_part_of_word() => {
                                while let Some(t) = get(caret.row, i) && t.is_part_of_word() {
                                    caret.col += t.width() as i32 * delta;
                                    i += delta;
                                }
                            }
                            Some(t) if t.is_punct() => {
                                while let Some(t) = get(caret.row, i) && t.is_punct() {
                                    caret.col += t.width() as i32 * delta;
                                    i += delta;
                                }
                            }
                            _ => {
                                // we're done
                            }
                        }
                    }
                    _ => unsafe { debug_unreachable!() },
                }
            }

            (_, Direction::Up) => {
                if caret.row <= 0 {
                    caret.col = 0;
                } else {
                    caret.row -= 1;
                    caret = self.snap(caret.with_col(desired_col.unwrap_or(caret.col)));
                }
            }
            (_, Direction::Down) => {
                if caret.row >= self.len() as i32 - 1 {
                    caret.col = self.line_width(self.len() as i32 - 1);
                } else {
                    caret.row += 1;
                    caret = self.snap(caret.with_col(desired_col.unwrap_or(caret.col)));
                }
            }
            (MoveVariant::ByToken, Direction::Right) => {
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
            (MoveVariant::ByToken, Direction::Left) => {
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
    pub fn move_selection_caret(
        &self,
        selection: &mut Selection,
        dir: Direction,
        selecting: bool,
        variant: MoveVariant,
    ) {
        let (caret, desired_col) =
            self.calculate_caret_move(selection.caret, selection.desired_col, dir, variant);

        selection.move_caret_to(caret, selecting);
        selection.desired_col = desired_col;
    }

    pub fn insert(&mut self, pos: Pos, data: LineData) -> InsertionInfo {
        debug_assert_eq!(pos, self.snap(pos));

        let data = data.0;

        let (r, i) = self.snap_indices(pos);

        let mut dcol = 0;

        if let Some(first_line) = data.first() {
            let multiline = data.len() > 1;

            let range = if multiline { i..self.0[r].len() } else { i..i };

            let split_off = self.0[r].splice(range, first_line.clone());

            if !multiline {
                dcol = first_line.iter().map(|cell| cell.width()).sum::<usize>() as i32;
            } else {
                let split_off: Vec<_> = split_off.collect();

                self.0.splice((r + 1)..(r + 1), data[1..].iter().cloned());

                let last_len = data.last().unwrap().len();

                let last_width = data
                    .last()
                    .unwrap()
                    .iter()
                    .map(|cell| cell.width())
                    .sum::<usize>() as i32;

                dcol = last_width - pos.col;

                self.0[r + data.len() - 1].splice(last_len..last_len, split_off);
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

        let (r_start, i) = self.snap_indices(start);
        let (r_end, j) = self.snap_indices(end);

        if start.row == end.row {
            self.0[r_start].splice(i..j, []);
        } else {
            let split_off: Vec<_> = self.0[r_end].splice(j.., []).collect();
            self.0[r_start].splice(i.., split_off);
            self.0.splice((r_start + 1)..(r_end + 1), []);
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

    pub fn copy_range(&self, Range { start, end }: Range) -> LineData {
        debug_assert_eq!(start, self.snap(start));
        debug_assert_eq!(end, self.snap(end));
        debug_assert!(start <= end);

        let (r_start, i) = self.snap_indices(start);
        let (r_end, j) = self.snap_indices(end);

        if start.row == end.row {
            return LineData(vec![self.0[r_start][i..j]
                .iter()
                .cloned()
                .collect::<Vec<_>>()]);
        } else {
            let mut lines = vec![];

            lines.push(self.0[r_start][i..].iter().cloned().collect::<Vec<_>>());

            for row in (r_start + 1)..(r_end) {
                lines.push(self.0[row].iter().cloned().collect::<Vec<_>>());
            }

            lines.push(self.0[r_end][..j].iter().cloned().collect::<Vec<_>>());

            return LineData(lines);
        }
    }

    // TODO -- search for multiline texts
    pub fn search_next_occurrence(&self, pos: Pos, text: &LineData) -> Option<Range> {
        assert_eq!(text.0.len(), 1);
        let tokens = &text.0[0];

        let (r0, r0i0) = self.snap_indices(pos);
        for r in r0..self.0.len() {
            let i0 = if r == r0 { r0i0 + 1 } else { 0 };
            'compare: for i in i0..self.0[r].len() {
                for j in 0..tokens.len() {
                    if i + j >= self.0[r].len() || tokens[j] != self.0[r][i + j] {
                        continue 'compare;
                    }
                }

                // found
                let row = r as i32;
                let col = self.line_index_col(row, i);
                return Some(Range {
                    start: Pos { row, col },
                    end: Pos {
                        row,
                        col: col + tokens.iter().map(|t| t.width()).sum::<usize>() as i32,
                    },
                });
            }
        }

        None
    }

    pub fn find_word_at(&self, pos: Pos) -> Option<Range> {
        let (pos, i, prev, next, _, _) = self.snap_nearest(pos);

        // let mut word_tokens = vec![];

        let mut start_i = i;
        let mut end_i = i;

        // prefer to select widget on the right, if possible
        if let Some(t) = next && t.is_widget() {
            return Some(Range {
                start: pos,
                end: Pos {
                    row: pos.row,
                    col: pos.col + t.width() as i32,
                },
            });
        }

        // selecting word going right
        if let Some(t) = next && t.is_part_of_word() {
            let mut i = i;
            while let Some(t) = self.0[pos.row as usize].get(i) && t.is_part_of_word() {
                // word_tokens.push(*t);
                end_i = i + 1;
                i += 1;
            }
        }

        // if no word on right, try to select widget on left
        if start_i == end_i && let Some(t) = prev && t.is_widget() {
            return Some(Range {
                start: Pos {
                    row: pos.row,
                    col: pos.col - t.width() as i32,
                },
                end: pos,
            });
        }

        // selecting word going left
        if let Some(t) = prev && t.is_part_of_word() {
            let mut i = i - 1;
            while let Some(t) = self.0[pos.row as usize].get(i) && t.is_part_of_word() {
                // word_tokens.insert(0, *t);
                start_i = i;
                if i == 0 {
                    break;
                }
                i -= 1;
            }
        }

        if start_i < end_i {
            // println!("found: [{} -- {}] {:?}", start_i, end_i, word_tokens);
            return Some(Range {
                start: Pos {
                    row: pos.row,
                    col: self.line_index_col(pos.row, start_i),
                },
                end: Pos {
                    row: pos.row,
                    col: self.line_index_col(pos.row, end_i),
                },
            });
        }

        // if word_tokens.len() > 0 {
        //     return Some(word_tokens.into());
        // }

        None
    }

    pub fn hover(&self, pos: Pos<f32>) -> Option<Token> {
        if pos.row < 0.0 || pos.col < 0.0 {
            return None;
        }

        let row = pos.row.floor() as usize;
        let Some(line) = self.0.get(row) else {
            return None;
        };

        let col = pos.col.floor() as usize;

        let mut acc = 0;
        for token in line {
            let w = token.width();
            if acc <= col && col < acc + w {
                return Some(*token);
            }
            acc += token.width();
        }

        None
    }

    /**
        Snaps the given pos to the neasest valid caret position, and returns:

        - the snapped/nearest valid caret position
        - the index in that row
        - the cell at the previous position (which can be `None` if at the start of a line);
        - the cell at that position (which can be `None` if at the end of a line);
        - whether the position was inside the text (end of line is valid);
        - whether the position was valid (i.e. not inside a widget);
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

    fn snap_indices(&self, pos: Pos) -> (usize, usize) {
        (pos.row as usize, self.snap_nearest(pos).1)
    }
}

impl ToString for LineData {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|line| {
                line.iter()
                    .map(|t| match t {
                        Token::Char(ch) => ch.to_string(),
                        Token::Widget { .. } => "[WIDGET]".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
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

impl From<String> for LineData {
    fn from(str: String) -> Self {
        Self::from(str.as_str())
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
