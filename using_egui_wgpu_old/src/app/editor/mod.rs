use egui::{epaint::*, *};

use crate::syntax_highlighting::code_view_ui;

pub struct Pos {
    pub row: i32,
    pub col: i32,
}

pub struct Selection {
    caret: Pos,
    anchor: Option<Pos>,
}

pub struct Editor {
    code: String,
    selections: Vec<Selection>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            code: "let kick = {\n    let env = envelope[a=5ms * bezier(.46,.1,.77,.47), d=50ms, s=400ms, r=400ms];\n    sin[40hz] * env\n};\n\nlet bpm = 120;\nlet beat = 60/bpm;\n\nlet hat = sample[\"/Users/kelley/emp/2022-11 Blabl Project/Samples/Processed/Freeze/Freeze RES [2022-11-23 221454].wav\"];\n\nlet house = kick * every(beat) + hat * (every(.5*beat) + .5*beat);\n\nplay house;".into(),
            selections: vec![],
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.vertical(|ui| {
                ui.add_space(20.0);
                code_view_ui(ui, &self.code);
                ui.add_space(20.0);
            });
            ui.add_space(20.0);
        });
    }
}
