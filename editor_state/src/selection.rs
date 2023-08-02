use std::cmp::Ordering;

use crate::{Direction, EditResult, InsertionInfo, Pos, Range, RemovalInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub id: usize,
    pub anchor: Option<Pos>,
    pub caret: Pos,
    pub desired_col: Option<i32>,
}

impl Selection {
    pub fn has_selection(&self) -> Option<Range> {
        self.anchor.and_then(|anchor| {
            if anchor == self.caret {
                None
            } else {
                Some(Pos::order(self.caret, anchor))
            }
        })
    }

    pub fn range(&self) -> Range {
        Pos::order(self.caret, self.anchor.unwrap_or(self.caret))
    }

    pub fn set_range(&mut self, range: Range, caret_on_left: bool) {
        if range.start == range.end {
            self.caret = range.start;
            self.anchor = None;
        } else if caret_on_left {
            self.caret = range.start;
            self.anchor = Some(range.end);
        } else {
            self.caret = range.end;
            self.anchor = Some(range.start);
        }
    }

    pub fn overlaps(&self, other: &Selection) -> bool {
        Range::overlap(self.range(), other.range())
    }

    pub fn adjust(&mut self, res: EditResult) {
        match res {
            EditResult::Insertion {
                info:
                    InsertionInfo {
                        start,
                        delta,
                        added_lines,
                        ..
                    },
            } => {
                if self.caret >= start {
                    if self.caret.row == start.row {
                        self.caret = self.caret + delta;

                        // this is very edge-casey, it would probably only occur if something other than user input would result in this insertion ... so maybe we should just remove it?
                        if let Some(col) = self.desired_col.as_mut() {
                            *col += delta.col;
                        }
                    } else {
                        self.caret.row += added_lines;
                    }
                }

                if let Some(anchor) = self.anchor.as_mut() {
                    if *anchor >= start {
                        if anchor.row == start.row {
                            *anchor = *anchor + delta;
                        } else {
                            anchor.row += added_lines;
                        }
                    }
                }
            }
            EditResult::Removal {
                info:
                    RemovalInfo {
                        start: _,
                        end,
                        delta,
                        removed_lines,
                    },
            } => {
                if self.caret >= end {
                    if self.caret.row == end.row {
                        self.caret = self.caret + delta;

                        // this is very edge-casey, it would probably only occur if something other than user input would result in this removal ... so maybe we should just remove it?
                        if let Some(col) = self.desired_col.as_mut() {
                            *col += delta.col;
                        }
                    } else {
                        self.caret.row -= removed_lines;
                    }
                }

                if let Some(anchor) = self.anchor.as_mut() {
                    if *anchor >= end {
                        if anchor.row == end.row {
                            *anchor = *anchor + delta;
                        } else {
                            anchor.row -= removed_lines;
                        }
                    }
                }
            }
        }
    }

    pub fn move_caret_to(&mut self, caret: Pos, selecting: bool) {
        if !selecting {
            self.anchor = None;
        } else if self.anchor == Some(caret) {
            // keep invariant: anchor != caret
            self.anchor = None;
        } else if self.anchor.is_none() {
            // anchor before move
            self.anchor = Some(self.caret);
        } else {
            // anchor was already set previously
        }

        self.caret = caret;
    }

    pub fn merge_with(&mut self, other: &Selection, prefer_caret_position: Option<Direction>) {
        let caret_on_left = prefer_caret_position
            .map(|dir| dir == Direction::Up || dir == Direction::Left)
            .unwrap_or(self.anchor.is_some_and(|anchor| anchor < self.caret));

        let cover = Range::cover(self.range(), other.range());
        self.set_range(cover, caret_on_left);

        // seems really edge-casey to try to save this one...
        self.desired_col = None;
    }
}

impl PartialOrd for Selection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Selection {
    // Orders component-wise: first by `caret`, then by `anchor`
    // Not that we really need this, because in the editor's selection set, selections will never  overlap --- this is just to make `Ord` work correctly in general
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.caret.cmp(&other.caret) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.anchor.cmp(&other.anchor),
        }
    }
}
