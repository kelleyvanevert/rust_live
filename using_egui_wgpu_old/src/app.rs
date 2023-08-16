use egui::{
    epaint::Shadow, hex_color, pos2, vec2, Context, FontFamily, FontId, Label, Layout, Rect,
    Response, RichText, Sense, Stroke, Ui, Vec2, Widget, WidgetText,
};

pub struct App<'a> {
    editors: Vec<&'a str>,
    current_editor: usize,
}

impl<'a> App<'a> {
    pub fn new(ctx: &Context) -> Self {
        setup_custom_fonts(ctx);

        Self {
            editors: vec![
                "exp_1.live".into(),
                "Untitled-1".into(),
                "Untitled-2".into(),
            ],
            current_editor: 0,
        }
    }

    pub fn ui(&mut self, ctx: &Context) {
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

                ui.add(SampleDash::new());

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        let text =  RichText::new("let kick = {\n    let env = envelope[a=5ms * bezier(.46,.1,.77,.47), d=50ms, s=400ms, r=400ms];\n    sin[40hz] * env\n};").monospace().size(18.0).color(hex_color!("#222222"));

                        ui.add(Label::new(text));
                     });
                    ui.add_space(20.0);
                });
            });
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

    // Put my font first (highest priority) for proportional text:
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec!["Plus Jakarta Medium".to_owned()],
    );

    fonts.families.insert(
        egui::FontFamily::Name("Bold".into()),
        vec!["Plus Jakarta Bold".to_owned()],
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

pub struct SampleDash {}

impl SampleDash {
    const HEIGHT: f32 = 256.0;

    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for SampleDash {
    fn ui(self, ui: &mut Ui) -> Response {
        let at = ui.cursor().left_top();
        let rect = Rect::from_min_max(at, at + vec2(f32::INFINITY, SampleDash::HEIGHT));

        ui.allocate_ui(vec2(f32::INFINITY, SampleDash::HEIGHT), |ui| {
            ui.set_min_height(SampleDash::HEIGHT);

            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, 0.0, hex_color!("#0B07C7"));

                ui.add_space(20.0);

                ui.horizontal(|ui| {
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

                ui.add_space(12.0);
            }
        })
        .response
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
