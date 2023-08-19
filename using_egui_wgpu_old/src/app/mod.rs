use egui::{epaint::*, *};

use self::{
    dash::{collapsed_ancestor_pane, Dash, DASH_HEIGHT},
    easing_dash::EasingDash,
    editor::Editor,
    envelope_dash::EnvelopeDash,
    sample_dash::SampleDash,
    session_dash::SessionDash,
    tab_button::TabButton,
};

mod dash;
mod easing_dash;
mod editor;
mod envelope_dash;
mod mini_button;
mod sample_dash;
mod session_dash;
mod tab_button;

pub struct EditorState {
    title: String,
    dash: Vec<Box<dyn Dash>>,
}

impl EditorState {
    #[allow(unused)]
    pub fn dash(&self) -> Option<&Box<dyn Dash>> {
        self.dash.last()
    }

    pub fn dash_mut(&mut self) -> Option<&mut Box<dyn Dash>> {
        self.dash.last_mut()
    }

    pub fn move_to_ancestor(&mut self, index: usize) {
        self.set_active(false);
        self.dash.splice(index + 1.., []);
        self.set_active(true);
    }

    pub fn set_active(&mut self, active: bool) {
        if let Some(dash) = self.dash_mut() {
            dash.set_active(active);
        }
    }
}

pub struct AppState {
    editors: Vec<EditorState>,
    current_editor: usize,
}

impl AppState {
    pub fn editor(&self) -> &EditorState {
        &self.editors[self.current_editor]
    }

    pub fn editor_mut(&mut self) -> &mut EditorState {
        &mut self.editors[self.current_editor]
    }

    pub fn switch_to_editor(&mut self, index: usize) {
        self.editor_mut().set_active(false);
        self.current_editor = index;
        self.editor_mut().set_active(true);
    }
}

enum StateUpdate {
    SwitchToEditor(usize),
    MoveToAncestor(usize),
}

pub struct App {
    state: AppState,
    updates: Vec<StateUpdate>,
    editor: Editor,
}

impl App {
    pub const WINDOW_DRAG_SURFACE_HEIGHT: usize = 54;

    pub fn new(ctx: &Context) -> Self {
        setup_custom_fonts(ctx);

        let mut editors = vec![
            EditorState {
                title: "exp_1.live".into(),
                dash: vec![
                    Box::new(SessionDash::new()),
                    Box::new(SampleDash::new(
                        "../editor/res/samples/Freeze RES [2022-11-23 221454].wav",
                    )),
                    Box::new(EnvelopeDash::new()),
                    Box::new(EasingDash::new()),
                ],
            },
            EditorState {
                title: "Untitled-1".into(),
                dash: vec![
                    Box::new(SessionDash::new()),
                    Box::new(SampleDash::new(
                        "../editor/res/samples/Freeze RES [2022-11-23 221454].wav",
                    )),
                ],
            },
            EditorState {
                title: "Untitled-2".into(),
                dash: vec![Box::new(SessionDash::new())],
            },
        ];

        let current_editor = 2;

        editors[current_editor].set_active(true);

        Self {
            state: AppState {
                editors,
                current_editor,
            },
            updates: vec![],
            editor: Editor::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        for update in self.updates.drain(0..) {
            match update {
                StateUpdate::SwitchToEditor(index) => {
                    self.state.switch_to_editor(index);
                }
                StateUpdate::MoveToAncestor(index) => {
                    self.state.editor_mut().move_to_ancestor(index);
                }
            }
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

                let header_height = 54.0;
                ui.allocate_ui_with_layout(
                    vec2(f32::INFINITY, header_height),
                    Layout {
                        main_dir: egui::Direction::LeftToRight,
                        main_wrap: false,
                        main_justify: false,
                        main_align: egui::Align::Min,
                        cross_justify: false,
                        cross_align: egui::Align::Center,
                    },
                    |ui| {
                        ui.set_min_height(header_height);

                        ui.spacing_mut().item_spacing = vec2(16.0, 0.0);
                        ui.add_space(92.);

                        let mut set_editor = None;
                        for (i, editor) in self.state.editors.iter().enumerate() {
                            let btn = ui.add(TabButton::new(
                                &editor.title,
                                self.state.current_editor == i,
                            ));
                            if btn.clicked() {
                                set_editor = Some(i);
                            }
                        }
                        if let Some(index) = set_editor {
                            self.updates.push(StateUpdate::SwitchToEditor(index));
                        }
                    },
                );

                ui.allocate_ui_with_layout(
                    vec2(f32::INFINITY, DASH_HEIGHT),
                    Layout {
                        main_dir: egui::Direction::LeftToRight,
                        main_wrap: false,
                        main_justify: false,
                        main_align: egui::Align::Min,
                        cross_justify: false,
                        cross_align: egui::Align::Center,
                    },
                    |ui| {
                        ui.set_min_height(DASH_HEIGHT);

                        let mut clicked = None;
                        for (i, dash) in self.state.editor().dash
                            [..self.state.editor().dash.len() - 1]
                            .iter()
                            .enumerate()
                        {
                            if collapsed_ancestor_pane(
                                ui,
                                dash.title(),
                                dash.title_color(),
                                dash.bg_color(),
                            )
                            .clicked()
                            {
                                clicked = Some(i);
                            }
                        }
                        if let Some(i) = clicked {
                            self.updates.push(StateUpdate::MoveToAncestor(i));
                        }

                        if let Some(last_dash) = self.state.editor_mut().dash.last_mut() {
                            last_dash.ui(ui);
                        }
                    },
                );

                ui.add_space(3.0); // ?? why?

                self.editor.ui(ui);
                // ScrollArea::vertical()
                //     .scroll_bar_visibility(scroll_area::ScrollBarVisibility::AlwaysVisible) // ?? doesn't show?
                //     .drag_to_scroll(false) // ?? doesn't work?
                //     .show(ui, |ui| {
                //         self.editor.ui(ui);
                //     });
            })
            .response
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "Plus Jakarta Bold".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/PlusJakartaSans-Bold.ttf"
        )),
    );

    fonts.font_data.insert(
        "Plus Jakarta Medium".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/PlusJakartaSans-Medium.ttf"
        )),
    );

    fonts.font_data.insert(
        "Fira Code".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/FiraCode-Retina.ttf")),
    );

    fonts.font_data.insert(
        "Fira Code Bold".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/FiraCode-Bold.ttf")),
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
        vec!["Fira Code Bold".to_owned()],
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
