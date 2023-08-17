use egui::*;

use super::dash::{Dash, DASH_HEIGHT};

pub struct SessionDash {}

impl SessionDash {
    pub fn new() -> Self {
        Self {}
    }
}

impl Dash for SessionDash {
    fn ui(&mut self, ui: &mut Ui) {
        let (response, painter) =
            ui.allocate_painter(vec2(f32::INFINITY, DASH_HEIGHT), Sense::click());

        let mut rect = response.rect;

        rect.max.x = ui.clip_rect().max.x;

        if !ui.is_rect_visible(rect) {
            return;
        }

        painter.rect_filled(rect, 0.0, self.bg_color());

        ui.allocate_ui_at_rect(
            Rect {
                min: rect.left_top() + vec2(20.0, 17.0),
                max: rect.left_top() + vec2(f32::INFINITY, 40.0),
            },
            |ui| {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.label(
                        RichText::new(self.title())
                            .size(18.0)
                            .family(FontFamily::Name("Bold".into()))
                            .color(self.title_color()),
                    );
                });
            },
        );
    }

    fn title(&self) -> String {
        "Live session".into()
    }

    fn title_color(&self) -> Color32 {
        hex_color!("#ffffff")
    }

    fn bg_color(&self) -> Color32 {
        hex_color!("#C7077A")
    }
}
