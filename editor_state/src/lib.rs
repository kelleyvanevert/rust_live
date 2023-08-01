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
            lines: LineData::from(
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
            .with_widget_at_pos(Pos { row: 6, col: 7 }, 1, 4)
            .with_inserted(Pos { row: 0, col: 5 }, LineData::from("hi\nthere kelley ")),
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

    pub fn file_drop(&mut self, pos: Pos, filepaths: Vec<PathBuf>) {
        // create widget
        let widget = SampleWidget {
            filepath: filepaths.first().unwrap().clone(),
        };
        let width = widget.width_in_editor();
        let id = self.widgets.len();
        self.widgets.push(Box::new(widget));

        // add to code & select
        let pos = self.lines.snap(pos);
        let (_, mut selection) = self.mk_selection(pos);
        self.lines.insert(pos, Cell::Widget { id, width }.into());
        self.lines
            .move_selection_caret(&mut selection, Direction::Right, false);
        self.selecting = None;

        self.selections = vec![selection];
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

    pub fn insert(&mut self, pos: Pos, data: LineData) {
        let pos = self.lines.snap(pos);
        let res = self.lines.insert(pos, data);

        for s in &mut self.selections {
            s.adjust(res);
        }
    }

    pub fn remove(&mut self, start: Pos, end: Pos) {
        let res = self.lines.remove(start, end);

        for s in &mut self.selections {
            s.adjust(res);
            // TODO remove selections that should no longer exist
        }
    }

    pub fn type_char(&mut self, ch: char) {
        for i in 0..self.selections.len() {
            if let Some((start, end)) = self.selections[i].has_selection() {
                self.remove(start, end);
                self.insert(start, LineData::from(ch));
            } else {
                self.insert(self.selections[i].caret, LineData::from(ch));
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
