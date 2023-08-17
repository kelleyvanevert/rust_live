use egui::{epaint::*, *};

use super::dash::{Dash, DASH_HEIGHT};

pub struct EnvelopeDash {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

impl EnvelopeDash {
    pub fn new() -> Self {
        Self {
            attack: 0.05,
            decay: 0.1,
            sustain: 0.4,
            release: 0.3,
        }
    }
}

impl Dash for EnvelopeDash {
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

        let env_rect = Rect {
            min: rect.min + vec2(60.0, 70.0),
            max: rect.max - vec2(60.0, 55.0),
        };

        // // debug
        // ui.painter()
        //     .rect_filled(env_rect, 0.0, hex_color!("#cc000077"));

        let w = env_rect.width();
        let h = env_rect.height();
        let Pos2 { x: xmin, y: ymin } = env_rect.left_top();
        let Pos2 { x: xmax, y: ymax } = env_rect.right_bottom();

        let start_pos = pos2(xmin, ymax);
        let end_pos = pos2(xmax, ymax);

        let total = self.attack + self.decay + self.sustain + self.release;
        let c1_pos = pos2(xmin + (self.attack / total) * w, ymin);
        let c2_pos = pos2(
            xmin + ((self.attack + self.decay) / total) * w,
            ymax - self.sustain * h,
        );
        let c3_pos = pos2(
            xmin + ((self.attack + self.decay + self.sustain) / total) * w,
            ymax - self.sustain * h,
        );

        let fat_stroke = Stroke::new(4.0, hex_color!("#000000"));

        let mut shapes = vec![];

        shapes.push(Shape::line_segment([start_pos, c1_pos], fat_stroke));
        shapes.push(Shape::line_segment([c1_pos, c2_pos], fat_stroke));
        shapes.push(Shape::line_segment([c2_pos, c3_pos], fat_stroke));
        shapes.push(Shape::line_segment([c3_pos, end_pos], fat_stroke));

        shapes.push(Shape::Circle(CircleShape {
            center: c1_pos,
            radius: 9.0,
            stroke: fat_stroke,
            fill: self.bg_color(),
        }));

        shapes.push(Shape::Circle(CircleShape {
            center: c2_pos,
            radius: 9.0,
            stroke: fat_stroke,
            fill: self.bg_color(),
        }));

        shapes.push(Shape::Circle(CircleShape {
            center: c3_pos,
            radius: 9.0,
            stroke: fat_stroke,
            fill: self.bg_color(),
        }));

        shapes.push(Shape::Circle(CircleShape {
            center: start_pos,
            radius: 9.0,
            stroke: fat_stroke,
            fill: self.bg_color(),
        }));

        shapes.push(Shape::Circle(CircleShape {
            center: end_pos,
            radius: 9.0,
            stroke: fat_stroke,
            fill: self.bg_color(),
        }));

        painter.extend(shapes);
    }

    fn title(&self) -> String {
        "Envelope".into()
    }

    fn title_color(&self) -> Color32 {
        hex_color!("#000000")
    }

    fn bg_color(&self) -> Color32 {
        hex_color!("#FFDB21")
    }
}
