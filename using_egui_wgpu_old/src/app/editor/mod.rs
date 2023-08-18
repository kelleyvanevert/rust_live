use egui::{epaint::*, *};

use crate::app::editor::line_data::MoveVariant;

use self::{
    direction::Direction,
    editor_state::{EditorState, LineSelection},
    highlight::{syntax_highlight, CodeToken},
    line_data::LineData,
    pos::Pos,
};

mod direction;
mod editor_state;
mod highlight;
mod line_data;
mod pos;
mod selection;

struct CodeTheme {
    keyword: TextFormat,
    text: TextFormat,
    // widget: TextFormat,
}

pub struct Editor {
    editor_state: EditorState,
    is_selecting: Option<usize>,

    char_size: Vec2,

    theme: CodeTheme,
}

impl Editor {
    pub fn new() -> Self {
        // let clipboard = Clipboard::new();

        // let mut widget_manager = WidgetManager::new();

        // let w0 = widget_manager.add(Box::new(SampleWidget::new(
        //     "./res/samples/Abroxis - Extended Oneshot 019.wav",
        // )));
        // let w1 = widget_manager.add(Box::new(SampleWidget::new("./res/samples/meii - Teag.wav")));

        let linedata = LineData::from(
            "let kick = {
    let env = envelope[a=5ms * bezier(.46,.1,.77,.47), d=50ms, s=400ms, r=400ms];
    sin[40hz] * env
};

let bpm = 120;
let beat = 60/bpm;

let hat = sample[\"/Users/kelley/emp/2022-11 Blabl Project/Samples/Processed/Freeze/Freeze RES [2022-11-23 221454].wav\"];

let house = kick * every(beat) + hat * (every(.5*beat) + .5*beat);

play house;",
        );
        // .with_widget_at_pos(Pos { row: 4, col: 40 }, w0)
        // .with_widget_at_pos(Pos { row: 6, col: 18 }, w1);

        let mut editor_state = EditorState::new().with_linedata(linedata);

        editor_state.add_caret(Pos { col: 3, row: 0 });

        let char_size = vec2(11.05, 24.0);

        let code_font_size = 18.0;

        let regular_font_id = FontId {
            size: code_font_size,
            family: egui::FontFamily::Monospace,
        };

        let bold_font_id = FontId {
            size: code_font_size,
            family: egui::FontFamily::Name("Code Bold".into()),
        };

        Self {
            // widget_manager,
            editor_state,
            char_size,
            // clipboard,
            is_selecting: None,
            // hovering_widget_id: None,
            // pressing_widget_id: None,
            //
            theme: CodeTheme {
                keyword: TextFormat {
                    font_id: bold_font_id.clone(),
                    color: hex_color!("#000000"),
                    ..Default::default()
                },
                text: TextFormat {
                    font_id: regular_font_id.clone(),
                    color: hex_color!("#333333"),
                    ..Default::default()
                },
                // widget: TextFormat {
                //     ..Default::default()
                // },
            },
        }
    }

    // fn find_widget(
    //     &self,
    //     renderer: &Renderer,
    //     mouse: (f32, f32),
    // ) -> Option<(usize, (f32, f32, f32, f32), (f32, f32))> {
    //     renderer.widget_at(mouse).map(|(id, quad)| {
    //         return (id, quad, mouse);
    //     })
    // }

    pub fn ui(&mut self, ui: &mut Ui) {
        let (response, painter) =
            ui.allocate_painter(vec2(f32::INFINITY, f32::INFINITY), Sense::click_and_drag());

        let rect = response.rect;

        let padding = vec2(20.0, 16.0);

        let pos_to_px = |pos: Pos| -> Pos2 {
            rect.left_top() + padding + self.char_size * vec2(pos.col as f32, pos.row as f32)
        };

        let px_to_pos = |xy: Pos2| -> Pos {
            let Vec2 { x, y } = (xy - rect.left_top() - padding) / self.char_size;
            Pos {
                row: y.floor() as i32,
                col: x.round() as i32,
            }
        };

        let mut shapes = vec![];

        // 1. DRAW CODE
        // ===

        let mk_keyword = |pos: Pos, text: String| -> TextShape {
            TextShape {
                pos: pos_to_px(pos),
                galley: ui.painter().layout(
                    text,
                    self.theme.keyword.font_id.clone(),
                    self.theme.keyword.color,
                    f32::INFINITY,
                ),
                underline: self.theme.keyword.underline,
                override_text_color: None,
                angle: 0.0,
            }
        };

        let mk_text = |pos: Pos, text: String| -> TextShape {
            TextShape {
                pos: pos_to_px(pos),
                galley: ui.painter().layout(
                    text,
                    self.theme.text.font_id.clone(),
                    self.theme.text.color,
                    f32::INFINITY,
                ),
                underline: self.theme.text.underline,
                override_text_color: None,
                angle: 0.0,
            }
        };

        // text_shapes.push(Shape::Text(mk_text(
        //     Pos { col: 0, row: 0 },
        //     "ABCDEFGHIJKL".into(),
        // )));
        // text_shapes.push(Shape::Text(mk_text(
        //     Pos { col: 4, row: 1 },
        //     "EFGHIJKL".into(),
        // )));
        // text_shapes.push(Shape::Text(mk_text(Pos { col: 8, row: 2 }, "IJKL".into())));

        for (row, line) in syntax_highlight(self.editor_state.linedata()) {
            for token in line {
                match token {
                    CodeToken::Keyword { text, col, .. } => {
                        shapes.push(Shape::Text(mk_keyword(
                            Pos {
                                col: col as i32,
                                row: row as i32,
                            },
                            text,
                        )));
                        //code_section.text.push(mk_keyword(text))
                    }
                    CodeToken::Text { text, col, .. } => {
                        shapes.push(Shape::Text(mk_text(
                            Pos {
                                col: col as i32,
                                row: row as i32,
                            },
                            text,
                        )));
                        //code_section.text.push(mk_regular(text))
                    }
                    CodeToken::Widget { col, width, id } => {
                        //code_section.text.push(mk_widget_space(width));

                        // let (x_start, y) = system.pos_to_px(Pos {
                        //     row: row as i32,
                        //     col: col as i32,
                        // });

                        // let (x_end, _) = system.pos_to_px(Pos {
                        //     row: row as i32,
                        //     col: (col + width) as i32,
                        // });

                        // widget_instances.push((
                        //     id,
                        //     (
                        //         x_start,
                        //         y + 4.0 / sf,
                        //         x_end,
                        //         y + system.char_size.1 / sf - 4.0 / sf,
                        //     ),
                        // ));
                    }
                }
            }

            // code_section.text.push(mk_regular("\n".into()));
        }

        // 2. DRAW SELECTIONS
        // ===

        for LineSelection {
            row,
            col_start,
            col_end,
        } in self.editor_state.visual_selections()
        {
            let Pos2 { x: x_start, y } = pos_to_px(Pos {
                row,
                col: col_start,
            });

            let Pos2 { x: x_end, y: _ } = pos_to_px(Pos { row, col: col_end });

            shapes.push(Shape::rect_filled(
                Rect {
                    min: pos2(x_start, y - 2.0),
                    max: pos2(x_end + 3.0, y + self.char_size.y - 2.0),
                },
                0.0,
                hex_color!("#00000033"),
            ));

            // builder.push_quad(
            //     x_start,
            //     y,
            //     x_end + 6.0 / sf,
            //     y + char_size.1 / sf,
            //     [0.0, 0.0, 0.0, 0.2],
            // );
        }

        for caret in self.editor_state.caret_positions() {
            let Pos2 { x, y } = pos_to_px(caret);

            shapes.push(Shape::rect_filled(
                Rect {
                    min: pos2(x, y - 2.0),
                    max: pos2(x + 3.0, y + self.char_size.y - 2.0),
                },
                0.0,
                hex_color!("#000000"),
            ));

            // builder.push_quad(
            //     x,
            //     y,
            //     x + 6.0 / sf,
            //     y + char_size.1 / sf,
            //     [0.0, 0.0, 0.0, 1.0],
            // );
        }

        painter.extend(shapes);

        // 3. PROCESS EVENTS
        // ===

        // [x] escape       => close
        // [x] tab(shift)
        // [not needed] space        => insert " ")
        // [x] enter        => insert "\n"
        // [x] backspace(alt?, cmd?)
        // [x] arrow up or down + cmd + alt
        // [x] arrow(shift?, alt?, cmd?)
        // [TODO] copy
        // [TODO] paste
        // [TODO] cut
        // [x] cmd+A
        // [x] cmd+D
        // [x] type text

        let alt = ui.input(|i| i.modifiers.alt);
        let shift = ui.input(|i| i.modifiers.shift);
        let cmd = ui.input(|i| i.modifiers.command);

        let arrow_up = ui.input(|i| i.key_pressed(Key::ArrowUp));
        let arrow_right = ui.input(|i| i.key_pressed(Key::ArrowRight));
        let arrow_down = ui.input(|i| i.key_pressed(Key::ArrowDown));
        let arrow_left = ui.input(|i| i.key_pressed(Key::ArrowLeft));
        let arrow = arrow_up || arrow_right || arrow_down || arrow_left;

        let interact_pos = ui.input(|i| i.pointer.interact_pos()).map(px_to_pos);

        if response.double_clicked() {
            if let Some(pos) = interact_pos {
                self.editor_state.select_word_at(pos);
            }
        } else if response.drag_started() {
            // .clicked() also works for clicks, but it's like MouseUp, whereas .drag_started() is like MouseDown
            if let Some(pos) = interact_pos {
                if shift {
                    if self.editor_state.has_selections() {
                        self.is_selecting = self.editor_state.extend_selection_to(pos);
                    } else {
                        self.is_selecting = Some(self.editor_state.set_single_caret(pos));
                    }
                } else if alt {
                    self.is_selecting = Some(self.editor_state.add_caret(pos));
                } else {
                    self.is_selecting = Some(self.editor_state.set_single_caret(pos));
                }
            }
        } else if ui.input_mut(|i| i.key_pressed(Key::Escape)) {
            println!("ESC");
        } else if ui.input_mut(|i| i.key_pressed(Key::Tab)) {
            if shift {
                self.editor_state.untab();
            } else {
                self.editor_state.tab();
            }
        } else if ui.input_mut(|i| i.key_pressed(Key::Enter)) {
            self.editor_state.write("\n");
        } else if ui.input_mut(|i| i.key_pressed(Key::Backspace)) {
            self.editor_state.backspace(if alt {
                MoveVariant::ByWord
            } else if cmd {
                MoveVariant::UntilEnd
            } else {
                MoveVariant::ByToken
            });
        } else if (arrow_up || arrow_down) && cmd && alt {
            self.editor_state.add_caret_vertically(if arrow_up {
                Direction::Up
            } else {
                Direction::Down
            });
        } else if arrow {
            self.editor_state.move_caret(
                if arrow_up {
                    Direction::Up
                } else if arrow_right {
                    Direction::Right
                } else if arrow_down {
                    Direction::Down
                } else {
                    Direction::Left
                },
                shift,
                if alt {
                    MoveVariant::ByWord
                } else if cmd {
                    MoveVariant::UntilEnd
                } else {
                    MoveVariant::ByToken
                },
            );
        } else if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::C)) {
            println!("copy");
        } else if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::X)) {
            println!("cut");
        } else if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::V)) {
            println!("paste");
        } else if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::A)) {
            self.editor_state.select_all();
        } else if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::D)) {
            self.editor_state.word_select();
        }

        // let events = ui.input(|i| i.events.clone());
        // if events.len() > 0 {
        //     println!("events: {:?}", events);
        // }

        if let Some(Event::Text(text)) = ui.input(|i| {
            i.events
                .iter()
                .find(|e| matches!(e, Event::Text(_)))
                .cloned()
        }) {
            self.editor_state.write(&text);
        }

        if response.dragged() && response.drag_delta() != Vec2::ZERO {
            if let Some(pos) = interact_pos && let Some(id) = self.is_selecting {
                self.editor_state.drag_select(pos, id);
             }
        }

        if response.drag_released() {
            self.is_selecting = None;
        }
    }
}
