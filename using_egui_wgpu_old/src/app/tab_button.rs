use egui::{epaint::*, *};

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
