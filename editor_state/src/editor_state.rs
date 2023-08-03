use std::collections::HashSet;

use tinyset::SetUsize;

use crate::{
    selection::Selection, Direction, EditResult, LineData, MoveVariant, Pos, Range, Token,
    WidgetInfo,
};

pub struct LineSelection {
    pub row: i32,
    pub col_start: i32,
    pub col_end: i32,
}

pub struct EditorState {
    linedata: LineData,
    pub tab_width: usize,
    next_selection_id: usize,
    selections: Vec<Selection>,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState {
            linedata: LineData::new(),
            tab_width: 2,
            next_selection_id: 0,
            selections: vec![],
        }
    }

    pub fn with_tab_width(mut self, tab_width: usize) -> Self {
        self.tab_width = tab_width;
        self
    }

    pub fn with_linedata(mut self, linedata: LineData) -> Self {
        self.linedata = linedata;
        self
    }

    /** Ensure that no two selections overlap */
    fn normalize_selections(
        &mut self,
        selecting_id: Option<usize>,
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

        normalized.sort_by_key(|s| s.caret);

        self.selections = normalized;
    }

    pub fn linedata(&self) -> &LineData {
        &self.linedata
    }

    pub fn caret_positions(&self) -> Vec<Pos> {
        self.selections.iter().map(|s| s.caret).collect()
    }

    pub fn has_selections(&self) -> bool {
        self.selections.len() > 0
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

    pub fn find_widget_at(&self, pos: Pos<f32>) -> Option<(usize, (f32, f32))> {
        match self.linedata.hover(pos) {
            Some((Token::Widget(WidgetInfo { id, .. }), p)) => Some((id, p)),
            _ => None,
        }
    }

    fn selection(&mut self) -> SelectionBuilder<NoCaret> {
        SelectionBuilder::new(self)
    }

    pub fn add_caret(&mut self, pos: Pos) -> usize {
        let caret = self.linedata.snap(pos);
        self.selection().caret(caret).add()
    }

    pub fn set_single_caret(&mut self, pos: Pos) -> usize {
        let caret = self.linedata.snap(pos);
        self.selection().caret(caret).set_only()
    }

    pub fn select_all(&mut self) -> usize {
        let end = self.linedata.end();
        self.selection()
            .for_range(Range {
                start: (0, 0).into(),
                end,
            })
            .set_only()
    }

    /**
        Perform "word selection", such as it will also typically happen in VS Code when pressing Cmd+D:

        - if there's a mismatch in selections (meaning that not all selections contain the same underlying text):
            - for each just-caret-selection, that neighbors a word: select the whole word
            - (otherwise, do nothing)

        - if there's no mismatch
            - if the match is '' (i.e. nothing selected), then for each just-caret-selection, that neighbors a word: select the whole word
            - otherwise, if there's another occurrence of whatever's currently selected, select the first one after the most recently added (or changed?) selection
    */
    pub fn word_select(&mut self) {
        if self.selections.len() == 0 {
            return; // nothing to do
        }

        // first, check if all selections match
        let text = self.linedata.copy_range(self.selections[0].range());
        let mismatch = self.selections[1..]
            .iter()
            .any(|s| self.linedata.copy_range(s.range()) != text);

        if mismatch || text.empty() {
            let mut done = SetUsize::new();
            while let Some(s) = self
                .selections
                .iter_mut()
                .find(|s| !done.contains(s.id) && s.just_caret())
            {
                done.insert(s.id);

                if let Some(range) = self.linedata.find_word_at(s.caret) {
                    s.anchor = Some(range.start);
                    s.caret = range.end;
                    s.desired_col = Some(range.start.col);
                }
            }

            self.normalize_selections(None, Some(Direction::Right));
        } else {
            let already_found = self
                .selections
                .iter()
                .map(|s| s.range())
                .collect::<HashSet<_>>();

            let (s, _) = self
                .selections
                .iter()
                .map(|s| (s, s.id))
                .max_by_key(|t| t.1)
                .unwrap();

            let mut search_from = s.range().end;
            loop {
                if let Some(found_range) = self.linedata.search_next_occurrence(search_from, &text)
                {
                    if already_found.contains(&found_range) {
                        search_from = found_range.end;
                        // continue search for next
                    } else {
                        self.selection().for_range(found_range).add();
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    // pub fn get_

    pub fn extend_selection_to(&mut self, pos: Pos) -> Option<usize> {
        let Some(first_selection_id) = self.selections.iter().map(|s| s.id).min() else {
            return None;
        };

        self.selections.retain_mut(|s| {
            if s.id == first_selection_id {
                s.desired_col = None;
                s.caret = self.linedata.snap(pos);
                true
            } else {
                false
            }
        });

        Some(first_selection_id)
    }

    pub fn copy(&self) -> Vec<LineData> {
        self.selections
            .iter()
            .filter(|s| s.anchor.is_some())
            .map(|s| self.linedata.copy_range(s.range()))
            .collect()
    }

    pub fn cut(&mut self) -> Vec<LineData> {
        let copied = self.copy();

        self.remove_selections();

        copied
    }

    pub fn paste(&mut self, mut data: Vec<LineData>) {
        if data.len() == 0 {
            return;
        }

        let num_sources = data.len();
        let num_targets = self.selections.len();

        if num_sources == num_targets {
            // easy, done!
        } else if num_sources == 1 {
            data = (0..num_targets)
                .map(|_| data[0].clone())
                .collect::<Vec<_>>();
        } else {
            data = (0..num_targets)
                .map(|_| LineData::joined(data.clone()))
                .collect::<Vec<_>>();
        }

        debug_assert_eq!(data.len(), num_targets);

        let mapping = data
            .into_iter()
            .zip(self.selections.iter().map(|s| s.id))
            .collect::<Vec<_>>();

        for (data, id) in mapping {
            let Some(s) = self.selections.iter().find(|s| s.id == id) else {
                continue;
            };

            if let Some(range) = s.has_selection() {
                self.remove(range);
                self.insert(range.start, data, false);
            } else {
                self.insert(s.caret, data, false);
            }
        }
    }

    pub fn file_drag_hover(&mut self, pos: Pos) {
        self.set_single_caret(pos);
    }

    pub fn drag_select(&mut self, caret: Pos, id: usize) {
        if let Some(s) = self.selections.iter_mut().find(|s| s.id == id) {
            s.move_caret_to(self.linedata.snap(caret), true);
        }

        self.normalize_selections(Some(id), None);
    }

    pub fn add_caret_vertically(&mut self, dir: Direction) {
        assert!(dir == Direction::Up || dir == Direction::Down);

        let mut carets_to_add = vec![];

        for s in &self.selections {
            let (caret, desired_col) = self.linedata.calculate_caret_move(
                s.caret,
                s.desired_col,
                dir,
                MoveVariant::ByToken,
            );

            carets_to_add.push((caret, desired_col));
        }

        for (caret, desired_col) in carets_to_add {
            self.selection()
                .caret(caret)
                .with_desired_col(desired_col)
                .add();
        }

        self.normalize_selections(None, Some(dir))
    }

    pub fn move_caret(&mut self, dir: Direction, selecting: bool, variant: MoveVariant) {
        for s in &mut self.selections {
            self.linedata
                .move_selection_caret(s, dir, selecting, variant);
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

    pub fn remove(&mut self, Range { start, end }: Range) {
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

        self.normalize_selections(None, None);
    }

    pub fn tab(&mut self) {
        let mut rows_selected = SetUsize::new();
        let mut regular_tabs = vec![];

        for s in &self.selections {
            if let Some(range) = s.has_selection() {
                for i in range.start.row..=range.end.row {
                    rows_selected.insert(i as usize);
                }
            } else {
                regular_tabs.push(s.id);
            }
        }

        for row in rows_selected {
            if self.linedata.line_empty(row) {
                continue;
            }

            let indent = self.linedata.line_indent(row);
            let add = ((indent as f32 / self.tab_width as f32).floor() as usize + 1)
                * self.tab_width
                - indent;

            self.insert(
                Pos {
                    row: row as i32,
                    col: indent as i32,
                },
                (0..add).map(|_| ' ').collect::<Vec<_>>().into(),
                false,
            );
        }

        for id in regular_tabs {
            let Some(s) = self.selections.iter().find(|s| s.id == id) else {
                continue;
            };

            self.insert(
                s.caret,
                (0..self.tab_width).map(|_| ' ').collect::<Vec<_>>().into(),
                false,
            );
        }
    }

    pub fn untab(&mut self) {
        let mut rows_selected = SetUsize::new();

        for s in &self.selections {
            let range = s.range();
            for i in range.start.row..=range.end.row {
                rows_selected.insert(i as usize);
            }
        }

        for row in rows_selected {
            let indent = self.linedata.line_indent(row);
            let new_indent = ((indent as f32 / self.tab_width as f32).ceil() as usize)
                .saturating_sub(1)
                * self.tab_width;

            self.remove(Range {
                start: Pos {
                    row: row as i32,
                    col: new_indent as i32,
                },
                end: Pos {
                    row: row as i32,
                    col: indent as i32,
                },
            });
        }
    }

    pub fn write(&mut self, text: &str) {
        let mut done = SetUsize::new();
        while let Some(s) = self.selections.iter().find(|s| !done.contains(s.id)) {
            done.insert(s.id);

            if let Some(range) = s.has_selection() {
                self.remove(range);
                self.insert(range.start, LineData::from(text), false);
            } else {
                self.insert(s.caret, LineData::from(text), false);
            }
        }
    }

    pub fn backspace(&mut self, variant: MoveVariant) {
        let mut done = SetUsize::new();
        while let Some(s) = self.selections.iter().find(|s| !done.contains(s.id)) {
            done.insert(s.id);

            if let Some(range) = s.has_selection() {
                self.remove(range);
            } else {
                let (prev_pos, _) = self.linedata.calculate_caret_move(
                    s.caret,
                    None,
                    Direction::Left,
                    if variant == MoveVariant::UntilEnd && s.caret.col == 0 {
                        // a little edge-case ;)
                        MoveVariant::ByToken
                    } else {
                        variant
                    },
                );

                self.remove(Range {
                    start: prev_pos,
                    end: s.caret,
                });
            }
        }
    }

    pub fn remove_selections(&mut self) {
        let mut done = SetUsize::new();
        while let Some(s) = self.selections.iter().find(|s| !done.contains(s.id)) {
            done.insert(s.id);

            self.remove(s.range());
        }
    }
}

struct Caret(Pos);
struct NoCaret;

struct SelectionBuilder<'a, C> {
    state: &'a mut EditorState,
    caret: C,
    anchor: Option<Pos>,
    desired_col: Option<i32>,
}

impl<'a> SelectionBuilder<'a, NoCaret> {
    fn new(state: &'a mut EditorState) -> Self {
        Self {
            state,
            caret: NoCaret,
            anchor: None,
            desired_col: None,
        }
    }
}

#[allow(unused)]
impl<'a, C> SelectionBuilder<'a, C> {
    fn with_anchor(mut self, anchor: Option<Pos>) -> Self {
        self.anchor = anchor;
        self
    }

    fn anchor(mut self, anchor: Pos) -> Self {
        self.anchor = Some(anchor);
        self
    }

    fn no_anchor(mut self) -> Self {
        self.anchor = None;
        self
    }

    fn with_desired_col(mut self, desired_col: Option<i32>) -> Self {
        self.desired_col = desired_col;
        self
    }

    fn desired_col(mut self, col: i32) -> Self {
        self.desired_col = Some(col);
        self
    }

    fn no_desired_col(mut self) -> Self {
        self.desired_col = None;
        self
    }
}

impl<'a> SelectionBuilder<'a, NoCaret> {
    fn for_range(self, range: Range) -> SelectionBuilder<'a, Caret> {
        let Self {
            state, desired_col, ..
        } = self;

        SelectionBuilder {
            state,
            anchor: Some(range.start),
            caret: Caret(range.end),
            desired_col,
        }
    }

    fn caret(self, caret: Pos) -> SelectionBuilder<'a, Caret> {
        let Self {
            state,
            anchor,
            desired_col,
            ..
        } = self;

        SelectionBuilder {
            state,
            anchor,
            caret: Caret(caret),
            desired_col,
        }
    }
}

impl<'a> SelectionBuilder<'a, Caret> {
    fn add(self) -> usize {
        let id = self.state.next_selection_id;

        let selection = Selection {
            id,
            caret: self.caret.0,
            anchor: self.anchor,
            desired_col: self.desired_col,
        };

        self.state.next_selection_id += 1;

        self.state.selections.push(selection);

        id
    }

    fn set_only(self) -> usize {
        self.state.selections = vec![];
        self.add()
    }
}
