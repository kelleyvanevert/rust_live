use egui::{epaint::*, *};

use super::{
    dash::{Dash, DASH_HEIGHT},
    mini_button::MiniButton,
};

#[derive(Debug, Clone)]
pub enum Easing {
    Linear,
    Quad(Pos2),
    Cubic(Pos2, Pos2),
    Smooth(Vec<Pos2>),
}

impl Easing {
    pub fn default_linear() -> Easing {
        Easing::Linear
    }

    pub fn default_quad() -> Easing {
        Easing::Quad(pos2(0.8, 0.1))
    }

    pub fn default_cubic() -> Easing {
        Easing::Cubic(pos2(0.4, 0.0), pos2(0.9, 0.3))
    }

    pub fn default_smooth() -> Easing {
        Easing::Smooth(vec![])
    }
}

pub struct EasingDash {
    easing: Easing,
}

impl EasingDash {
    pub fn new() -> Self {
        Self {
            easing: Easing::Cubic(pos2(0.4, 0.0), pos2(0.9, 0.3)),
        }
    }
}

impl Dash for EasingDash {
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

        let margin = 50.0;

        let easing_rect = Rect {
            min: rect.min + vec2(150.0, margin),
            max: rect.max - vec2(100.0, margin),
        };

        let max_move_rect = easing_rect.expand(margin * 0.7);

        // // debug
        // ui.painter()
        //     .rect_filled(easing_rect, 0.0, hex_color!("#cc000077"));

        let w = easing_rect.width();
        let h = easing_rect.height();
        let Pos2 { x: xmin, y: ymin } = easing_rect.left_top();
        let Pos2 { x: xmax, y: ymax } = easing_rect.right_bottom();

        let a_pos = pos2(xmin, ymax);
        let b_pos = pos2(xmax, ymin);

        let fat_stroke = Stroke::new(4.0, hex_color!("#000000"));

        let mut shapes = vec![];

        match &self.easing.clone() {
            Easing::Linear => {
                shapes.push(Shape::line_segment([a_pos, b_pos], fat_stroke));
            }
            Easing::Quad(cp) => {
                let cp_pos = a_pos + cp.to_vec2() * vec2(w, -h);

                shapes.push(Shape::QuadraticBezier(
                    QuadraticBezierShape::from_points_stroke(
                        [a_pos, cp_pos, b_pos],
                        false,
                        Color32::TRANSPARENT,
                        fat_stroke,
                    ),
                ));

                shapes.extend(Shape::dashed_line(&[a_pos, cp_pos], fat_stroke, 4.0, 4.0));

                shapes.extend(Shape::dashed_line(&[b_pos, cp_pos], fat_stroke, 4.0, 4.0));

                {
                    let cp_rect = Rect::from_center_size(cp_pos, vec2(20.0, 20.0));
                    let cp_id = response.id.with(0);
                    let cp_response = ui.interact(cp_rect, cp_id, Sense::drag());

                    if cp_response.drag_delta() != Vec2::ZERO {
                        let hover_pos = ui.input(|i| i.pointer.hover_pos());

                        let new_cp_pos = hover_pos
                            .unwrap_or(cp_pos)
                            .clamp(max_move_rect.min, max_move_rect.max)
                            - easing_rect.min;

                        let cp = pos2(new_cp_pos.x / w, 1.0 - new_cp_pos.y / h);

                        self.easing = Easing::Quad(cp);
                    }
                }

                shapes.push(Shape::Circle(CircleShape {
                    center: cp_pos,
                    radius: 8.0,
                    stroke: fat_stroke,
                    fill: self.bg_color(),
                }));
            }
            Easing::Cubic(c1, c2) => {
                let c1_pos = a_pos + c1.to_vec2() * vec2(w, -h);
                let c2_pos = a_pos + c2.to_vec2() * vec2(w, -h);

                shapes.push(Shape::CubicBezier(CubicBezierShape::from_points_stroke(
                    [a_pos, c1_pos, c2_pos, b_pos],
                    false,
                    Color32::TRANSPARENT,
                    fat_stroke,
                )));

                shapes.extend(Shape::dashed_line(&[a_pos, c1_pos], fat_stroke, 4.0, 4.0));

                shapes.extend(Shape::dashed_line(&[b_pos, c2_pos], fat_stroke, 4.0, 4.0));

                for (i, &cp_pos) in [c1_pos, c2_pos].iter().enumerate() {
                    let cp_rect = Rect::from_center_size(cp_pos, vec2(20.0, 20.0));
                    let cp_id = response.id.with(i);
                    let cp_response = ui.interact(cp_rect, cp_id, Sense::drag());

                    if cp_response.drag_delta() != Vec2::ZERO {
                        let hover_pos = ui.input(|i| i.pointer.hover_pos());

                        let new_cp_pos = hover_pos
                            .unwrap_or(cp_pos)
                            .clamp(max_move_rect.min, max_move_rect.max)
                            - easing_rect.min;

                        let cp = pos2(new_cp_pos.x / w, 1.0 - new_cp_pos.y / h);

                        if i == 0 {
                            self.easing = Easing::Cubic(cp, *c2);
                        } else {
                            self.easing = Easing::Cubic(*c1, cp);
                        }
                    }

                    shapes.push(Shape::Circle(CircleShape {
                        center: cp_pos,
                        radius: 8.0,
                        stroke: fat_stroke,
                        fill: self.bg_color(),
                    }));
                }
            }
            Easing::Smooth(_) => {
                // TODO
            }
        }

        shapes.push(Shape::Circle(CircleShape {
            center: a_pos,
            radius: 9.0,
            stroke: fat_stroke,
            fill: self.bg_color(),
        }));

        shapes.push(Shape::Circle(CircleShape {
            center: b_pos,
            radius: 9.0,
            stroke: fat_stroke,
            fill: self.bg_color(),
        }));

        painter.extend(shapes);

        ui.allocate_ui_at_rect(
            Rect {
                min: rect.left_top() + vec2(20.0, 66.0),
                max: rect.left_top() + vec2(100.0, 200.0),
            },
            |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                    if ui
                        .add(MiniButton::new(
                            "linear",
                            matches!(self.easing, Easing::Linear),
                        ))
                        .clicked()
                    {
                        self.easing = Easing::default_linear();
                    }

                    if ui
                        .add(MiniButton::new(
                            "quad",
                            matches!(self.easing, Easing::Quad(_)),
                        ))
                        .clicked()
                    {
                        self.easing = Easing::default_quad();
                    }

                    if ui
                        .add(MiniButton::new(
                            "bezier",
                            matches!(self.easing, Easing::Cubic(_, _)),
                        ))
                        .clicked()
                    {
                        self.easing = Easing::default_cubic();
                    }

                    if ui
                        .add(MiniButton::new(
                            "smooth",
                            matches!(self.easing, Easing::Smooth(_)),
                        ))
                        .clicked()
                    {
                        self.easing = Easing::default_smooth();
                    }
                });
            },
        );
    }

    fn title(&self) -> String {
        "Easing".into()
    }

    fn title_color(&self) -> Color32 {
        hex_color!("#000000")
    }

    fn bg_color(&self) -> Color32 {
        hex_color!("#F8B711")
    }
}
