use std::f32::consts::PI;

use egui::{epaint::TextShape, Color32, FontFamily, FontId, Response, Sense, Stroke, Ui};
use emath::vec2;

pub const DASH_HEIGHT: f32 = 256.0;

pub fn collapsed_ancestor_pane(
    ui: &mut Ui,
    title: impl Into<String>,
    title_color: Color32,
    bg_color: Color32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(40.0, DASH_HEIGHT), Sense::click());

    ui.painter().rect_filled(rect, 0.0, bg_color);

    let galley = ui.painter().layout(
        title.into(),
        FontId {
            size: 18.0,
            family: FontFamily::Name("Bold".into()),
        },
        title_color,
        f32::INFINITY,
    );

    ui.painter().add(TextShape {
        pos: rect.left_top() + vec2(31.0, 20.0),
        galley,
        angle: 0.5 * PI,
        underline: Stroke::NONE,
        override_text_color: None,
    });

    response
}

pub trait Dash {
    fn ui(&mut self, ui: &mut Ui);
    fn title(&self) -> String;
    fn title_color(&self) -> Color32;
    fn bg_color(&self) -> Color32;
}
