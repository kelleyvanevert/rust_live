use super::{
    direction::Direction,
    line_data::{EditResult, LineData},
    pos::Pos,
    selection::Selection,
};

pub struct LineSelection {
    pub row: i32,
    pub col_start: i32,
    pub col_end: i32,
}

pub struct EditorState {
    lines: LineData,

    /** ID of the currently selecting selection */
    selecting: Option<usize>,
    /** Selection ID increment */
    next_selection_id: usize,
    // TODO: enforce invariant: no overlap (and also not immediately adjacent on same line)
    selections: Vec<Selection>,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState {
            lines: LineData::new(),
            selecting: None,
            next_selection_id: 0,
            selections: vec![],
        }
    }

    pub fn with_linedata(mut self, linedata: LineData) -> Self {
        self.lines = linedata;
        self
    }

    // fn normalize_selections(&mut self) {
    //     let mut at: Pos = (0, 0).into();

    //     // self.selections = self.selections.sort_by(|a, b| a.ordered().0)
    // }

    // fn add_selection(&mut self, selection: Selection) {
    //     //
    // }

    pub fn linedata(&self) -> &LineData {
        &self.lines
    }

    pub fn caret_positions(&self) -> Vec<Pos> {
        self.selections.iter().map(|s| s.caret).collect()
    }

    pub fn visual_selections(&self) -> Vec<LineSelection> {
        let mut line_selections = vec![];

        for s in &self.selections {
            if let Some((start, end)) = s.has_selection() {
                if start.row == end.row {
                    line_selections.push(LineSelection {
                        row: start.row,
                        col_start: start.col.min(end.col),
                        col_end: start.col.max(end.col),
                    });
                } else if start.row < end.row {
                    line_selections.push(LineSelection {
                        row: start.row,
                        col_start: start.col,
                        col_end: self.lines.line_width(start.row),
                    });
                    for row in (start.row + 1)..end.row {
                        line_selections.push(LineSelection {
                            row,
                            col_start: 0,
                            col_end: self.lines.line_width(row),
                        });
                    }
                    line_selections.push(LineSelection {
                        row: end.row,
                        col_start: 0,
                        col_end: end.col,
                    });
                }
            }
        }

        line_selections
    }

    fn mk_selection(&mut self, caret: Pos) -> (usize, Selection) {
        debug_assert_eq!(caret, self.lines.snap(caret));

        let id = self.next_selection_id;

        let selection = Selection {
            id,
            caret,
            anchor: None,
            desired_col: None,
        };

        self.next_selection_id += 1;

        (id, selection)
    }

    pub fn add_caret(&mut self, pos: Pos) {
        let pos = self.lines.snap(pos);
        let (id, selection) = self.mk_selection(pos);
        self.selecting = Some(id);

        self.selections.push(selection);
    }

    pub fn set_single_caret(&mut self, pos: Pos) {
        let pos = self.lines.snap(pos);
        let (id, selection) = self.mk_selection(pos);
        self.selecting = Some(id);

        self.selections = vec![selection];
    }

    pub fn file_drag_hover(&mut self, pos: Pos) {
        self.set_single_caret(pos);
    }

    pub fn drag_select(&mut self, caret: Pos) {
        if let Some(id) = self.selecting && let Some(s) = self.selections.iter_mut().find(|s| s.id == id) {
            s.move_caret_to(self.lines.snap(caret), true);
        }
    }

    pub fn move_caret(&mut self, dir: Direction, selecting: bool) {
        for s in &mut self.selections {
            self.lines.move_selection_caret(s, dir, selecting);
        }
    }

    pub fn clear(&mut self) {
        self.lines = LineData::new()
    }

    pub fn insert(&mut self, pos: Pos, data: LineData, set_single_caret_after: bool) {
        let pos = self.lines.snap(pos);
        let info = self.lines.insert(pos, data);

        if set_single_caret_after {
            self.set_single_caret(info.end);
        } else {
            for s in &mut self.selections {
                s.adjust(EditResult::Insertion { info });
            }
        }
    }

    pub fn remove(&mut self, start: Pos, end: Pos) {
        let info = self.lines.remove(start, end);

        for s in &mut self.selections {
            s.adjust(EditResult::Removal { info });
            // TODO remove selections that should no longer exist
        }
    }

    pub fn type_char(&mut self, ch: char) {
        for i in 0..self.selections.len() {
            if let Some((start, end)) = self.selections[i].has_selection() {
                self.remove(start, end);
                self.insert(start, LineData::from(ch), false);
            } else {
                self.insert(self.selections[i].caret, LineData::from(ch), false);
            }
        }
    }

    pub fn backspace(&mut self) {
        for i in 0..self.selections.len() {
            if let Some((start, end)) = self.selections[i].has_selection() {
                self.remove(start, end);
            } else {
                let (prev_pos, _) = self.lines.calculate_caret_move(
                    self.selections[i].caret,
                    None,
                    Direction::Left,
                );

                self.remove(prev_pos, self.selections[i].caret);
            }
        }
    }
}
