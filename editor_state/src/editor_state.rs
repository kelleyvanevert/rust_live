use std::collections::HashSet;

use super::{
    direction::Direction,
    line_data::{EditResult, LineData},
    pos::{Pos, Range},
    selection::{Selection, SelectionId},
};

pub struct LineSelection {
    pub row: i32,
    pub col_start: i32,
    pub col_end: i32,
}

pub struct EditorState {
    linedata: LineData,

    next_selection_id: SelectionId,
    selections: Vec<Selection>,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState {
            linedata: LineData::new(),
            next_selection_id: SelectionId::start(),
            selections: vec![],
        }
    }

    pub fn with_linedata(mut self, linedata: LineData) -> Self {
        self.linedata = linedata;
        self
    }

    /** Ensure that no two selections overlap */
    fn normalize_selections(
        &mut self,
        selecting_id: Option<SelectionId>,
        prefer_caret_position: Option<Direction>,
    ) {
        let mut normalized = vec![];

        while let Some(mut next) = self.selections.pop() {
            self.selections.retain(|other| {
                if next.overlaps(other) {
                    if selecting_id == Some(next.id) {
                        // just kill the other
                        // (noop)
                    } else if selecting_id == Some(other.id) {
                        // just kill self
                        next = other.clone();
                    } else {
                        next.merge_with(other, prefer_caret_position);
                    }

                    return false;
                }

                return true;
            });

            normalized.push(next);
        }

        self.selections = normalized;
    }

    pub fn linedata(&self) -> &LineData {
        &self.linedata
    }

    pub fn caret_positions(&self) -> Vec<Pos> {
        self.selections.iter().map(|s| s.caret).collect()
    }

    pub fn visual_selections(&self) -> Vec<LineSelection> {
        let mut line_selections = vec![];

        for s in &self.selections {
            if let Some(Range { start, end }) = s.has_selection() {
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
                        col_end: self.linedata.line_width(start.row),
                    });
                    for row in (start.row + 1)..end.row {
                        line_selections.push(LineSelection {
                            row,
                            col_start: 0,
                            col_end: self.linedata.line_width(row),
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

    fn mk_selection(&mut self, caret: Pos) -> (SelectionId, Selection) {
        debug_assert_eq!(caret, self.linedata.snap(caret));

        let id = self.next_selection_id;

        let selection = Selection {
            id,
            caret,
            anchor: None,
            desired_col: None,
        };

        self.next_selection_id = id.next();

        (id, selection)
    }

    pub fn add_caret(&mut self, pos: Pos) -> SelectionId {
        let pos = self.linedata.snap(pos);
        let (id, selection) = self.mk_selection(pos);

        self.selections.push(selection);
        self.normalize_selections(Some(id), None); // ??

        id
    }

    pub fn set_single_caret(&mut self, pos: Pos) -> SelectionId {
        let pos = self.linedata.snap(pos);
        let (id, selection) = self.mk_selection(pos);

        self.selections = vec![selection];

        id
    }

    pub fn file_drag_hover(&mut self, pos: Pos) {
        self.set_single_caret(pos);
    }

    pub fn drag_select(&mut self, caret: Pos, id: SelectionId) {
        if let Some(s) = self.selections.iter_mut().find(|s| s.id == id) {
            s.move_caret_to(self.linedata.snap(caret), true);
        }

        self.normalize_selections(Some(id), None);
    }

    pub fn move_caret(&mut self, dir: Direction, selecting: bool) {
        for s in &mut self.selections {
            self.linedata.move_selection_caret(s, dir, selecting);
        }

        self.normalize_selections(None, Some(dir))
    }

    pub fn clear(&mut self) {
        self.linedata = LineData::new()
    }

    pub fn insert(&mut self, pos: Pos, data: LineData, set_single_caret_after: bool) {
        let pos = self.linedata.snap(pos);
        let info = self.linedata.insert(pos, data);

        if set_single_caret_after {
            self.set_single_caret(info.end);
        } else {
            for s in &mut self.selections {
                s.adjust(EditResult::Insertion { info });
            }
        }
    }

    pub fn remove(&mut self, start: Pos, end: Pos) {
        self.selections.retain(|s| {
            let contained_entirely = start < s.caret
                && s.caret < end
                && s.anchor
                    .map(|anchor| start < anchor && anchor < end)
                    .unwrap_or(true);

            !contained_entirely
        });

        let info = self.linedata.remove(start, end);

        for s in &mut self.selections {
            s.adjust(EditResult::Removal { info });
        }

        self.normalize_selections(None, None)
    }

    pub fn type_char(&mut self, ch: char) {
        let mut done: HashSet<SelectionId> = HashSet::new();
        while let Some(s) = self.selections.iter().find(|s| !done.contains(&s.id)) {
            done.insert(s.id);

            if let Some(Range { start, end }) = s.has_selection() {
                self.remove(start, end);
                self.insert(start, LineData::from(ch), false);
            } else {
                self.insert(s.caret, LineData::from(ch), false);
            }
        }
    }

    pub fn backspace(&mut self) {
        let mut done: HashSet<SelectionId> = HashSet::new();
        while let Some(s) = self.selections.iter().find(|s| !done.contains(&s.id)) {
            done.insert(s.id);

            if let Some(Range { start, end }) = s.has_selection() {
                self.remove(start, end);
            } else {
                let (prev_pos, _) =
                    self.linedata
                        .calculate_caret_move(s.caret, None, Direction::Left);

                self.remove(prev_pos, s.caret);
            }
        }
    }
}
