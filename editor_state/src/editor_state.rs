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
    lines: LineData,

    /** Selection ID increment */
    next_selection_id: SelectionId,
    // TODO: enforce invariant: no overlap (and also not immediately adjacent on same line)
    selections: Vec<Selection>,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState {
            lines: LineData::new(),
            next_selection_id: SelectionId::start(),
            selections: vec![],
        }
    }

    pub fn with_linedata(mut self, linedata: LineData) -> Self {
        self.lines = linedata;
        self
    }

    fn normalize_selections(
        &mut self,
        selecting_id: Option<SelectionId>,
        prefer_caret_position: Option<Direction>,
    ) {
        // (TODO make sure to not remove the one that is being selected)

        let mut cloned: Vec<_> = self.selections.drain(0..).collect();

        while let Some(mut next) = cloned.pop() {
            cloned.retain(|other| {
                if !next.overlaps(other) {
                    return true;
                }

                if selecting_id == Some(next.id) {
                    // just kill the other
                    // (noop)
                } else if selecting_id == Some(other.id) {
                    // just kill self
                    next = other.clone();
                } else {
                    let merged = next.merge_with(other, prefer_caret_position);
                    if !merged {
                        println!("WTH");
                    }
                }

                return false;
            });

            self.selections.push(next);
        }

        // let mut arr: Vec<_> = self
        //     .selections
        //     .drain(0..)
        //     .into_iter()
        //     .map(|s| (s.range(), s))
        //     .collect();

        // arr.sort_by_key(|t| t.0.start);

        // while let Some((range, sel)) = arr.pop() {
        //     while let Some(b) = arr.pop() {
        //         //
        //     }

        //     self.selections.push(sel);
        // }

        // if let Some((mut range_a, mut a)) = arr.pop() {
        //     for (mut range_b, mut b) in arr {
        //         if Range::overlap(range_a, range_b) {
        //             //
        //         }

        //         self.selections.push(a);
        //     }
        //     // while let Some((mut range_b, mut b)) = arr.pop() {
        //     //     if Range::overlap(range_a, range_b) {
        //     //         // if self.selecting == Some(b.id) {
        //     //         //     // just skip selection A
        //     //         // } else {
        //     //         // }

        //     //         // continue;
        //     //     }

        //     //     self.selections.push(a);
        //     //     a = b;
        //     //     range_a = range_b;
        //     // }

        //     self.selections.push(a);
        // }

        // let arr: Vec<(Range, Selection)> = arr.into_iter().fold(
        //     vec![],
        //     |mut normalized, (curr_range, mut curr_selection)| {
        //         let prev = normalized.pop();

        //         if let Some(prev) = prev {
        //             if !curr_selection.overlaps(&prev.1) {
        //                 normalized.push(prev);
        //             } else {
        //                 if self.selecting == Some(curr_selection.id) {
        //                     // nothing, just kill the other
        //                 } else {
        //                     let merged =
        //                         curr_selection.maybe_merge_with(&prev.1, prefer_caret_position);
        //                     if merged {
        //                         println!("merged");
        //                         if self.selecting == Some(prev.1.id) {
        //                             self.selecting = Some(curr_selection.id);
        //                         }
        //                     }
        //                 }
        //             }
        //         }

        //         normalized.push((curr_range, curr_selection));

        //         normalized
        //     },
        // );

        // self.selections = arr.into_iter().map(|t| t.1).collect();
    }

    pub fn linedata(&self) -> &LineData {
        &self.lines
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

    fn mk_selection(&mut self, caret: Pos) -> (SelectionId, Selection) {
        debug_assert_eq!(caret, self.lines.snap(caret));

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
        let pos = self.lines.snap(pos);
        let (id, selection) = self.mk_selection(pos);

        self.selections.push(selection);
        self.normalize_selections(Some(id), None); // ??

        id
    }

    pub fn set_single_caret(&mut self, pos: Pos) -> SelectionId {
        let pos = self.lines.snap(pos);
        let (id, selection) = self.mk_selection(pos);

        self.selections = vec![selection];

        id
    }

    pub fn file_drag_hover(&mut self, pos: Pos) {
        self.set_single_caret(pos);
    }

    pub fn drag_select(&mut self, caret: Pos, id: SelectionId) {
        if let Some(s) = self.selections.iter_mut().find(|s| s.id == id) {
            s.move_caret_to(self.lines.snap(caret), true);
        }

        self.normalize_selections(Some(id), None);
    }

    pub fn move_caret(&mut self, dir: Direction, selecting: bool) {
        for s in &mut self.selections {
            self.lines.move_selection_caret(s, dir, selecting);
        }

        self.normalize_selections(None, Some(dir))
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
        self.selections.retain(|s| {
            let contained_entirely = start < s.caret
                && s.caret < end
                && s.anchor
                    .map(|anchor| start < anchor && anchor < end)
                    .unwrap_or(true);

            !contained_entirely
        });

        let info = self.lines.remove(start, end);

        for s in &mut self.selections {
            s.adjust(EditResult::Removal { info });
        }

        self.normalize_selections(None, None)
    }

    pub fn type_char(&mut self, ch: char) {
        for i in 0..self.selections.len() {
            if let Some(Range { start, end }) = self.selections[i].has_selection() {
                self.remove(start, end);
                self.insert(start, LineData::from(ch), false);
            } else {
                self.insert(self.selections[i].caret, LineData::from(ch), false);
            }
        }
    }

    pub fn backspace(&mut self) {
        for i in 0..self.selections.len() {
            if let Some(Range { start, end }) = self.selections[i].has_selection() {
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
