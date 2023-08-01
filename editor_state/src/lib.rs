#![feature(let_chains)]

pub mod direction;
pub mod line_data;
pub mod pos;
mod selection;
mod widgets;

use std::path::PathBuf;

use self::{
    direction::Direction,
    line_data::{Cell, LineData},
    pos::Pos,
    selection::Selection,
    widgets::{sample::SampleWidget, Widget},
};

pub struct LineSelection {
    pub row: i32,
    pub col_start: i32,
    pub col_end: i32,
}

pub enum Token {
    Keyword { col: usize, text: String },
    Text { col: usize, text: String },
    Widget { col: usize, id: usize, width: usize },
}

pub struct EditorState {
    lines: LineData,

    widgets: Vec<Box<dyn Widget>>,

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
            widgets: vec![
                Box::new(SampleWidget {
                    filepath: PathBuf::new(),
                }),
                Box::new(SampleWidget {
                    filepath: PathBuf::new(),
                }),
            ],
            lines: LineData::from_str(
                "A kelley wrote
  some
  code that' eventually
  and bla bla bla bla bla

run off
  the screen
  and bla bla bla
",
            )
            .with_widget_at_pos(Pos { row: 2, col: 12 }, 0, 5)
            .with_widget_at_pos(Pos { row: 6, col: 7 }, 1, 4),
            selecting: None,
            next_selection_id: 0,
            selections: vec![],
        }
    }

    // fn normalize_selections(&mut self) {
    //     let mut at: Pos = (0, 0).into();

    //     // self.selections = self.selections.sort_by(|a, b| a.ordered().0)
    // }

    // fn add_selection(&mut self, selection: Selection) {
    //     //
    // }

    pub fn caret_positions(&self) -> Vec<Pos> {
        self.selections.iter().map(|s| s.caret).collect()
    }

    pub fn visual_selections(&self) -> Vec<LineSelection> {
        let mut line_selections = vec![];

        for s in &self.selections {
            if let Some((start, end)) = s.has_selection(&self.lines) {
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
        let id = self.next_selection_id;

        let selection = Selection {
            id,
            caret: self.lines.snap(caret).0,
            anchor: None,
            desired_col: None,
        };

        self.next_selection_id += 1;

        (id, selection)
    }

    pub fn add_caret(&mut self, pos: Pos) {
        let (id, selection) = self.mk_selection(pos);
        self.selecting = Some(id);

        self.selections.push(selection);
    }

    pub fn set_single_caret(&mut self, pos: Pos) {
        let (id, selection) = self.mk_selection(pos);
        self.selecting = Some(id);

        self.selections = vec![selection];
    }

    pub fn file_drag_hover(&mut self, pos: Pos) {
        self.set_single_caret(pos);
    }

    pub fn file_drop(&mut self, pos: Pos, filepaths: Vec<PathBuf>) {
        // create widget
        let widget = SampleWidget {
            filepath: filepaths.first().unwrap().clone(),
        };
        let width = widget.width_in_editor();
        let id = self.widgets.len();
        self.widgets.push(Box::new(widget));

        // add to code & select
        let (_, mut selection) = self.mk_selection(pos);
        self.lines.insert(pos, vec![Cell::Widget { id, width }]);
        self.lines
            .move_selection_caret(&mut selection, Direction::Right, false);
        self.selecting = None;

        self.selections = vec![selection];
    }

    pub fn drag_select(&mut self, caret: Pos) {
        if let Some(id) = self.selecting && let Some(s) = self.selections.iter_mut().find(|s| s.id == id) {
            s.move_caret_to(self.lines.snap(caret).0, true);
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

    pub fn type_char(&mut self, ch: char) {
        // // first, normalize selections:
        // // - no "visual but not logical" caret positions
        // // - type at start of selections, whether that's the caret or the anchor
        // for s in &mut self.selections {
        //     s.caret = s.caret.text_bounded(&self.lines);
        //     if let Some(anchor) = s.anchor {
        //         if anchor.row < s.caret.row {
        //             s.caret = anchor;
        //         } else if anchor.col < s.caret.col {
        //             s.caret = anchor;
        //         }
        //     }
        //     s.anchor = None;
        // }

        // for i in 0..self.selections.len() {
        //     self.type_char_at_selection(i, ch);
        // }
    }

    fn type_char_at_selection(&mut self, i: usize, ch: char) {
        // debug_assert!(i < self.selections.len());
        // let Pos { row, col } = self.selections[i].caret;

        // debug_assert!(row >= 0 && row < self.lines.len() as i32);

        // if ch != '\n' {
        //     // TODO: remove selection first, if there is one

        //     // the easy case
        //     let line = &mut self.lines[row as usize];
        //     debug_assert!(col >= 0 && col < line.len() as i32);

        //     line.insert(col as usize, ch);
        //     self.selections[i].caret.col += 1;
        // } else {
        //     // the harder case
        //     let after = self.lines[row as usize].split_off(col as usize);
        //     self.lines.insert(row as usize + 1, after);

        //     for s in &mut self.selections {
        //         if s.caret.row == row && s.caret.col == col {
        //             s.caret.row += 1;
        //             s.caret.col = 0;
        //         } else if s.caret.row == row && s.caret.col > col {
        //             s.caret.row += 1;
        //             s.caret.col -= col;
        //         } else if s.caret.row > row {
        //             s.caret.row += 1;
        //         }
        //     }
        // }
    }

    pub fn backspace(&mut self) {
        // // first, normalize selections:
        // // - no "visual but not logical" caret positions
        // for s in &mut self.selections {
        //     s.caret = s.caret.text_bounded(&self.lines);
        // }

        // for i in 0..self.selections.len() {
        //     self.backspace_at_selection(i);
        // }

        // // TODO: normalize selections for no-overlap invariant
    }

    fn backspace_at_selection(&mut self, i: usize) {
        // debug_assert!(i < self.selections.len());

        // let Selection { anchor, caret, .. } = self.selections[i];
        // let Pos { row, col } = caret;

        // debug_assert!(row >= 0 && row < self.lines.len() as i32);

        // if let Some(anchor) = anchor && anchor != caret {
        //     let (first, last) = Pos::order(anchor, caret);
        //     self.remove_selection(first, last);
        // } else if col > 0 {
        //     // remove a single character (the easy case)
        //     // [x] DONE

        //     self.lines[row as usize].remove(col as usize - 1);
        //     for s in &mut self.selections {
        //         // move back self + others in same line
        //         if s.caret.row == row && s.caret.col >= col {
        //             s.caret.col -= 1;
        //         }
        //         if let Some(anchor) = s.anchor.as_mut() && anchor.row == row && anchor.col >= col {
        //             anchor.col -= 1;
        //         }
        //     }
        // } else if row > 0 {
        //     // remove a newline (the slightly harder case)
        //     // [x] DONE

        //     let mut removed_line = self.lines.remove(row as usize);
        //     let prev_line_len = self.lines.line_width(row - 1);
        //     self.lines[row as usize - 1].append(&mut removed_line);
        //     for s in &mut self.selections {
        //         // move back self + others in same line
        //         if s.caret.row >= row {
        //             if s.caret.row == row {
        //                 s.caret.col += prev_line_len;
        //             }
        //             s.caret.row -= 1;
        //         }
        //         if let Some(anchor) = s.anchor.as_mut() && anchor.row >= row {
        //             if anchor.row == row {
        //                 anchor.col += prev_line_len;
        //             }
        //             anchor.row -= 1;
        //         }
        //     }
        // } else {
        //     // noop
        // }
    }

    fn remove_selection(&mut self, start: Pos, end: Pos) {
        // // remove a selection (the hardest case)
        // // self.selections[i] = Selection::from(first);

        // if start.row == end.row {
        //     self.lines[start.row as usize].splice((start.col as usize)..(end.col as usize + 1), []);
        // } else {
        //     let removed_from_end = self.lines[end.row as usize]
        //         .splice((end.col as usize).., [])
        //         .collect::<Vec<_>>();
        //     self.lines[start.row as usize].splice((start.col as usize).., removed_from_end);
        //     self.lines
        //         .splice((start.row as usize + 1)..(end.row as usize + 1), []);
        // }

        // for s in &mut self.selections {
        //     // TODO fix?
        //     if s.caret.within(start, end) {
        //         s.caret = start;
        //     }
        //     if let Some(anchor) = s.anchor.as_mut() && anchor.within(start, end) {
        //         *anchor = start;
        //     }
        // }

        // // TODO normalize selections
    }

    pub fn tokenize(&self) -> Vec<(usize, Vec<Token>)> {
        self.lines
            .lines()
            .iter()
            .map(|line| {
                let mut col = 0;

                let mut tokens: Vec<Token> = vec![];

                let mut space: String = "".into();
                let mut word: String = "".into();

                for &cell in line.iter() {
                    match cell {
                        Cell::Widget { id, width } => {
                            if word.len() > 0 {
                                let is_keyword = &word == "kelley";

                                tokens.push(if is_keyword {
                                    Token::Keyword { col, text: word }
                                } else {
                                    Token::Text { col, text: word }
                                });

                                word = "".into();
                            }

                            if space.len() > 0 {
                                tokens.push(Token::Text { col, text: space });

                                space = "".into();
                            }

                            tokens.push(Token::Widget { col, id, width });
                        }
                        Cell::Char(ch) => {
                            if ch == ' ' {
                                if word.len() > 0 {
                                    let is_keyword = &word == "kelley";

                                    tokens.push(if is_keyword {
                                        Token::Keyword { col, text: word }
                                    } else {
                                        Token::Text { col, text: word }
                                    });

                                    word = "".into();
                                }

                                space.push(ch);
                            } else {
                                if space.len() > 0 {
                                    tokens.push(Token::Text { col, text: space });

                                    space = "".into();
                                }

                                word.push(ch);
                            }
                        }
                    }

                    col += cell.width();
                }

                if word.len() > 0 {
                    let is_keyword = &word == "kelley";

                    tokens.push(if is_keyword {
                        Token::Keyword { col, text: word }
                    } else {
                        Token::Text { col, text: word }
                    });
                }

                if space.len() > 0 {
                    tokens.push(Token::Text { col, text: space });
                }

                // tokens.push(Token::Text("\n".into()));

                tokens
            })
            .enumerate()
            .collect()
    }
}
