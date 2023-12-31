use std::{cell::RefCell, f32::consts::PI, time::Instant};

use egui::{
    epaint::{CircleShape, CubicBezierShape, QuadraticBezierShape, Shadow, TextShape},
    hex_color, pos2, vec2, Color32, Context, FontFamily, FontId, Label, Layout, Pos2, Rect,
    Response, RichText, Sense, Shape, Stroke, Ui, Vec2, Widget, WidgetText,
};

use crate::{
    read_audio_file::{read_audio_file, AudioTrackInfo},
    syntax_highlighting::code_view_ui,
};

pub struct App<'a> {
    editors: Vec<&'a str>,
    current_editor: usize,
    sample_dash: SampleDash,
    easing_dash: EasingDash,
}

impl<'a> App<'a> {
    pub const WINDOW_DRAG_SURFACE_HEIGHT: usize = 54;

    pub fn new(ctx: &Context) -> Self {
        setup_custom_fonts(ctx);

        Self {
            editors: vec![
                "exp_1.live".into(),
                "Untitled-1".into(),
                "Untitled-2".into(),
            ],
            current_editor: 1,
            sample_dash: SampleDash::new(
                vec![(
                    "Live session".into(),
                    hex_color!("#ffffff"),
                    hex_color!("#C7077A"),
                )],
                "../editor/res/samples/Freeze RES [2022-11-23 221454].wav",
            ),
            easing_dash: EasingDash::new(vec![
                (
                    "Live session".into(),
                    hex_color!("#ffffff"),
                    hex_color!("#C7077A"),
                ),
                (
                    "Audio clip".into(),
                    hex_color!("#ffffff"),
                    hex_color!("#0B07C7"),
                ),
                (
                    "Envelope".into(),
                    hex_color!("#000000"),
                    hex_color!("#FFDB21"),
                ),
            ]),
        }
    }

    pub fn ui(&mut self, ctx: &Context) -> Response {
        let my_frame = egui::containers::Frame {
            inner_margin: 0.0.into(),
            outer_margin: 0.0.into(),
            rounding: 0.0.into(),
            shadow: Shadow::NONE,
            fill: hex_color!("#FAF9F8"),
            stroke: egui::Stroke::NONE,
        };

        egui::CentralPanel::default()
            .frame(my_frame)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;

                ui.allocate_ui_with_layout(
                    vec2(f32::INFINITY, 54.0),
                    Layout {
                        main_dir: egui::Direction::LeftToRight,
                        main_wrap: false,
                        main_justify: false,
                        main_align: egui::Align::LEFT,
                        cross_justify: false,
                        cross_align: egui::Align::Center,
                    },
                    |ui| {
                        ui.spacing_mut().item_spacing = vec2(16.0, 0.0);
                        ui.add_space(92.);

                        for (i, &filename) in self.editors.iter().enumerate() {
                            let btn = ui.add(TabButton::new(filename, self.current_editor == i));
                            if btn.clicked() {
                                self.current_editor = i;
                            }
                        }
                        // ui.add(TabButton::new("exp_1.live",true));
                        // ui.add(TabButton::new("Untitled-1",false));
                        // ui.add(TabButton::new("Untitled-2",false));
                    },
                );

                match self.current_editor {
                    0 => self.sample_dash.ui(ui),
                    _ => self.easing_dash.ui(ui),
                };

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        code_view_ui(ui, "let kick = {\n    let env = envelope[a=5ms * bezier(.46,.1,.77,.47), d=50ms, s=400ms, r=400ms];\n    sin[40hz] * env\n};");
                        // let text =  RichText::new("let kick = {\n    let env = envelope[a=5ms * bezier(.46,.1,.77,.47), d=50ms, s=400ms, r=400ms];\n    sin[40hz] * env\n};").monospace().size(18.0).color(hex_color!("#222222"));

                        // ui.add(Label::new(text));
                     });
                    ui.add_space(20.0);
                });
            }).response
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "Plus Jakarta Bold".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/PlusJakartaSans-Bold.ttf")),
    );

    fonts.font_data.insert(
        "Plus Jakarta Medium".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/PlusJakartaSans-Medium.ttf")),
    );

    fonts.font_data.insert(
        "Fira Code".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/FiraCode-Retina.ttf")),
    );

    fonts.font_data.insert(
        "Fira Code SemiBold".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/FiraCode-SemiBold.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec!["Plus Jakarta Medium".to_owned()],
    );

    fonts.families.insert(
        egui::FontFamily::Name("Bold".into()),
        vec!["Plus Jakarta Bold".to_owned()],
    );

    fonts.families.insert(
        egui::FontFamily::Name("Code Bold".into()),
        vec!["Fira Code SemiBold".to_owned()],
    );

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "Fira Code".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

const DASH_HEIGHT: f32 = 256.0;

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
    ancestors: Vec<(String, Color32, Color32)>,
    easing: Easing,
}

impl EasingDash {
    pub fn new(ancestors: Vec<(String, Color32, Color32)>) -> Self {
        Self {
            ancestors,
            easing: Easing::Cubic(pos2(0.4, 0.0), pos2(0.9, 0.3)),
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        let at = ui.cursor().left_top();
        let rect = Rect::from_min_max(at, at + vec2(f32::INFINITY, DASH_HEIGHT));

        let bg = hex_color!("#F8B711");

        ui.allocate_ui(vec2(f32::INFINITY, DASH_HEIGHT), |ui| {
            ui.set_min_height(DASH_HEIGHT);

            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, 0.0, bg);

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    let mut ancestor_pane = rect.clone();
                    ancestor_pane.set_width(40.0);

                    for (i, (ancestor_title, title_color, bg_color)) in
                        self.ancestors.iter().enumerate()
                    {
                        ui.painter().rect_filled(ancestor_pane, 0.0, *bg_color);

                        let galley = ui.painter().layout(
                            ancestor_title.into(),
                            FontId {
                                size: 18.0,
                                family: FontFamily::Name("Bold".into()),
                            },
                            *title_color,
                            f32::INFINITY,
                        );

                        ui.painter().add(TextShape {
                            pos: pos2(30.0 + i as f32 * 40.0, 54.0 + 20.0),
                            galley,
                            angle: 0.5 * PI,
                            underline: Stroke::NONE,
                            override_text_color: None,
                        });

                        ancestor_pane.min.x += 40.0;
                        ancestor_pane.max.x += 40.0;
                    }

                    ui.add_space(40.0 * self.ancestors.len() as f32);

                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Easing")
                            .family(FontFamily::Name("Bold".into()))
                            .color(hex_color!("#000000"))
                            .size(18.0),
                    );
                });

                let (response, painter) =
                    ui.allocate_painter(vec2(f32::INFINITY, DASH_HEIGHT - 50.0), Sense::drag());

                let le = self.ancestors.len() as f32 * 40.0;

                let paint_left_top = rect.left_top() + vec2(le + 20.0, 50.0);

                let paint_rect = Rect::from_min_max(
                    paint_left_top,
                    paint_left_top
                        + vec2(
                            ui.clip_rect().width() - le - 20.0 - 20.0,
                            DASH_HEIGHT - 50.0 - 30.0,
                        ),
                );

                // debug
                // ui.painter()
                //     .rect_filled(paint_rect, 0.0, hex_color!("#cc000077"));

                let mut easing_rect = paint_rect.clone();
                easing_rect.min += vec2(170.0, 20.0);
                easing_rect.max -= vec2(120.0, 20.0);
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
                                let d =
                                    cp_response.drag_delta() / easing_rect.size() * vec2(1.0, -1.0);
                                let cp = (*cp + d).clamp(pos2(-0.2, -0.2), pos2(1.2, 1.2));

                                self.easing = Easing::Quad(cp);
                            }
                        }

                        shapes.push(Shape::Circle(CircleShape {
                            center: cp_pos,
                            radius: 8.0,
                            stroke: fat_stroke,
                            fill: bg,
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

                        for (i, &cp) in [c1_pos, c2_pos].iter().enumerate() {
                            let cp_rect = Rect::from_center_size(cp, vec2(20.0, 20.0));
                            let cp_id = response.id.with(i);
                            let cp_response = ui.interact(cp_rect, cp_id, Sense::drag());

                            if cp_response.drag_delta() != Vec2::ZERO {
                                let d =
                                    cp_response.drag_delta() / easing_rect.size() * vec2(1.0, -1.0);
                                let cp = (if i == 0 { *c1 } else { *c2 } + d)
                                    .clamp(pos2(-0.2, -0.2), pos2(1.2, 1.2));

                                if i == 0 {
                                    self.easing = Easing::Cubic(cp, *c2);
                                } else {
                                    self.easing = Easing::Cubic(*c1, cp);
                                }
                            }

                            shapes.push(Shape::Circle(CircleShape {
                                center: cp,
                                radius: 8.0,
                                stroke: fat_stroke,
                                fill: bg,
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
                    fill: bg,
                }));

                shapes.push(Shape::Circle(CircleShape {
                    center: b_pos,
                    radius: 9.0,
                    stroke: fat_stroke,
                    fill: bg,
                }));

                painter.extend(shapes);

                ui.allocate_ui_at_rect(
                    Rect {
                        min: paint_left_top + vec2(0.0, 20.0),
                        max: paint_left_top + vec2(60.0, 200.0),
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
        });
    }
}

pub struct SampleDash {
    ancestors: Vec<(String, Color32, Color32)>,
    audio_file: AudioTrackInfo,
    width: usize,
    summary: RefCell<Option<Summary>>,
}

struct Summary {
    overall_max: f32,
    samples_overview: Vec<(f32, f32, f32)>,
}

impl SampleDash {
    pub fn new(ancestors: Vec<(String, Color32, Color32)>, filepath: &str) -> Self {
        let width = 0;
        let audio_file = read_audio_file(filepath);

        Self {
            ancestors,
            audio_file,
            width,
            summary: RefCell::new(None),
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        let at = ui.cursor().left_top();
        let rect = Rect::from_min_max(at, at + vec2(f32::INFINITY, DASH_HEIGHT));

        ui.allocate_ui(vec2(f32::INFINITY, DASH_HEIGHT), |ui| {
            ui.set_min_height(DASH_HEIGHT);

            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, 0.0, hex_color!("#0B07C7"));

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    let mut ancestor_pane = rect.clone();
                    ancestor_pane.set_width(40.0);

                    for (i, (ancestor_title, title_color, bg_color)) in
                        self.ancestors.iter().enumerate()
                    {
                        ui.painter().rect_filled(ancestor_pane, 0.0, *bg_color);

                        let galley = ui.painter().layout(
                            ancestor_title.into(),
                            FontId {
                                size: 18.0,
                                family: FontFamily::Name("Bold".into()),
                            },
                            *title_color,
                            f32::INFINITY,
                        );

                        ui.painter().add(TextShape {
                            pos: pos2(30.0 + i as f32 * 40.0, 54.0 + 20.0),
                            galley,
                            angle: 0.5 * PI,
                            underline: Stroke::NONE,
                            override_text_color: None,
                        });

                        ancestor_pane.min.x += 40.0;
                        ancestor_pane.max.x += 40.0;
                    }

                    ui.add_space(40.0 * self.ancestors.len() as f32);

                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Sample")
                            .family(FontFamily::Name("Bold".into()))
                            .color(hex_color!("#ffffff"))
                            .size(18.0),
                    );

                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Length: 2.3s")
                            .color(hex_color!("#ffffff66"))
                            .size(12.0),
                    );

                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Stereo")
                            .color(hex_color!("#ffffff66"))
                            .size(12.0),
                    );
                });

                ui.ctx().request_repaint();

                let paint_left_top = rect.left_top() + vec2(60.0, 50.0);

                let paint_rect = Rect::from_min_max(
                    paint_left_top,
                    paint_left_top
                        + vec2(
                            ui.clip_rect().width() - 60.0 - 20.0,
                            DASH_HEIGHT - 50.0 - 30.0,
                        ),
                );

                let width = paint_rect.width() as usize / 2;
                if width != self.width {
                    self.width = width;

                    println!("update");
                    let t0 = Instant::now();

                    let num_samples = self.audio_file.samples.len();
                    // physical pixels, btw
                    let samples_per_pixel = num_samples / width;

                    // (min, max, rms)
                    let mut samples_overview: Vec<(f32, f32, f32)> = vec![];

                    let (mut overall_min, mut overall_max) = (0.0, 0.0);
                    let (mut min, mut max) = (0.0, 0.0);

                    let mut count = 0;
                    let mut rms_range = vec![];

                    fn calculate_rms(samples: &Vec<f32>) -> f32 {
                        let sqr_sum = samples.iter().fold(0.0, |sqr_sum, s| {
                            let sample = *s as f32;
                            sqr_sum + sample * sample
                        });

                        (sqr_sum / samples.len() as f32).sqrt()
                    }

                    for i in 0..num_samples {
                        let sample = self.audio_file.samples[i];
                        rms_range.push(sample);

                        if sample < min {
                            min = sample;
                        }
                        if sample > max {
                            max = sample;
                        }
                        if sample < overall_min {
                            overall_min = sample;
                        }
                        if sample > overall_max {
                            overall_max = sample;
                        }

                        count += 1;
                        if count == samples_per_pixel {
                            let rms = calculate_rms(&rms_range);
                            // println!("[min ={} max= {}, rms = {}]", min, max, rms);
                            samples_overview.push((min, max, rms));
                            count = 0;
                            min = 0.0;
                            max = 0.0;
                            rms_range = vec![];
                        }
                    }

                    println!("Processed samples, took: {:?}", Instant::elapsed(&t0));

                    let _ = self.summary.borrow_mut().insert(Summary {
                        overall_max: overall_max.max(-overall_min),
                        samples_overview,
                    });
                }

                let mut shapes = vec![];

                // shapes.push(egui::epaint::Shape::Rect(egui::epaint::RectShape {
                //     rect: paint_rect,
                //     rounding: 0.0.into(),
                //     fill: hex_color!("#00000055"),
                //     stroke: Stroke::NONE,
                // }));

                let summary = self.summary.borrow();
                let summary = summary.as_ref().unwrap();

                let height = paint_rect.height();
                let half = height / 2.0;
                let scale = 0.85 * half * (1.0 / summary.overall_max);
                let y0 = paint_rect.min.y + half;

                for (i, &(min, max, rms)) in summary.samples_overview.iter().enumerate() {
                    let x = 2.0 * i as f32 + 60.0;
                    shapes.push(Shape::line_segment(
                        [pos2(x, y0 + min * scale), pos2(x, y0 + max * scale)],
                        Stroke::new(1.2, hex_color!("#ffffff77")),
                    ));
                    shapes.push(Shape::line_segment(
                        [pos2(x, y0 - rms * scale), pos2(x, y0 + rms * scale)],
                        Stroke::new(2.0, hex_color!("#ffffffff")),
                    ));
                }

                // let time = ui.input(|i| i.time);

                // let to_screen = emath::RectTransform::from_to(
                //     Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                //     paint_rect,
                // );

                // for &mode in &[2, 3, 5] {
                //     let mode = mode as f64;
                //     let n = 120;
                //     let speed = 1.5;

                //     let points: Vec<Pos2> = (0..=n)
                //         .map(|i| {
                //             let t = i as f64 / (n as f64);
                //             let amp = (time * speed * mode).sin() / mode;
                //             let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
                //             to_screen * pos2(t as f32, y as f32)
                //         })
                //         .collect();

                //     let thickness = 10.0 / mode as f32;
                //     shapes.push(egui::epaint::Shape::line(
                //         points,
                //         Stroke::new(thickness, hex_color!("#ffffff")),
                //     ));
                // }

                ui.painter().extend(shapes);
            }
        });
    }
}

pub struct TabButton {
    text: WidgetText,
    selected: bool,
}

impl TabButton {
    pub fn new(text: impl Into<WidgetText>, selected: bool) -> Self {
        Self {
            text: text.into(),
            selected,
        }
    }
}

impl Widget for TabButton {
    fn ui(self, ui: &mut Ui) -> Response {
        // Widget code can be broken up in four steps:
        //  1. Decide a size for the widget
        //  2. Allocate space for it
        //  3. Handle interactions with the widget (if any)
        //  4. Paint the widget

        let padding = vec2(24.0, 6.0);

        // 1. Deciding widget size:
        // You can query the `ui` how much space is available,
        // but in this example we have a fixed size widget based on the height of a standard button:

        let text = self.text.into_galley(
            ui,
            Some(false),
            ui.available_width() - 2. * padding.x,
            FontId {
                size: 14.,
                family: FontFamily::Name("Bold".into()),
            },
        );

        let desired_size = vec2(text.size().x + 2. * padding.x, 32.);

        // 2. Allocating space:
        // This is where we get a region of the screen assigned.
        // We also tell the Ui to sense clicks in the allocated region.
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

        // // 3. Interact: Time to check for clicks!
        // if response.clicked() {
        //     *selected = !*selected;
        //     response.mark_changed(); // report back that the value changed
        // }

        // Attach some meta-data to the response which can be used by screen readers:
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, text.text()));

        if ui.is_rect_visible(rect) {
            // let visuals = ui.style().interact(&response);

            ui.painter().rect(
                rect, //.expand(visuals.expansion)
                rect.height() / 2.0,
                if self.selected {
                    hex_color!("#000000")
                } else {
                    hex_color!("#ECECEC")
                },
                Stroke::NONE,
            );

            let text_pos = ui
                .layout()
                .align_size_within_rect(text.size(), rect.shrink2(padding))
                .min;

            text.paint_with_color_override(
                ui.painter(),
                text_pos,
                if self.selected {
                    hex_color!("#ffffff")
                } else {
                    hex_color!("#363636")
                },
            );
        }

        response
    }
}

pub struct MiniButton {
    text: WidgetText,
    selected: bool,
}

impl MiniButton {
    pub fn new(text: impl Into<WidgetText>, selected: bool) -> Self {
        Self {
            text: text.into(),
            selected,
        }
    }
}

impl Widget for MiniButton {
    fn ui(self, ui: &mut Ui) -> Response {
        // Widget code can be broken up in four steps:
        //  1. Decide a size for the widget
        //  2. Allocate space for it
        //  3. Handle interactions with the widget (if any)
        //  4. Paint the widget

        let padding = vec2(10.0, 4.0);

        // 1. Deciding widget size:
        // You can query the `ui` how much space is available,
        // but in this example we have a fixed size widget based on the height of a standard button:

        let text = self.text.into_galley(
            ui,
            Some(false),
            ui.available_width() - 2. * padding.x,
            FontId {
                size: 14.,
                family: FontFamily::Name("Bold".into()),
            },
        );

        let desired_size = vec2(text.size().x + 2.0 * padding.x, 20.);

        // 2. Allocating space:
        // This is where we get a region of the screen assigned.
        // We also tell the Ui to sense clicks in the allocated region.
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

        // // 3. Interact: Time to check for clicks!
        // if response.clicked() {
        //     *selected = !*selected;
        //     response.mark_changed(); // report back that the value changed
        // }

        // Attach some meta-data to the response which can be used by screen readers:
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, text.text()));

        if ui.is_rect_visible(rect) {
            // let visuals = ui.style().interact(&response);

            ui.painter().rect(
                rect, //.expand(visuals.expansion)
                4.0,
                if self.selected {
                    hex_color!("#00000066")
                } else {
                    hex_color!("#00000022")
                },
                Stroke::NONE,
            );

            let text_pos = ui
                .layout()
                .align_size_within_rect(text.size(), rect.shrink2(padding))
                .min;

            text.paint_with_color_override(
                ui.painter(),
                text_pos,
                if self.selected {
                    hex_color!("#000000")
                } else {
                    hex_color!("#333333")
                },
            );
        }

        response
    }
}
