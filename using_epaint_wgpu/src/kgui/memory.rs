use emath::Rect;

use super::{context::InputState, id::IdMap, input::RawInput};

#[derive(Clone, Debug, Default)]
pub struct Memory {
    pub options: Options,
    pub(crate) new_font_definitions: Option<epaint::text::FontDefinitions>,
    /// new scale that will be applied at the start of the next frame
    pub(crate) new_pixels_per_point: Option<f32>,
}

impl Memory {
    pub(crate) fn begin_frame(&mut self, prev_input: &InputState, new_input: &RawInput) {
        // self.interaction.begin_frame(prev_input, new_input);

        // if !prev_input.pointer.any_down() {
        //     self.window_interaction = None;
        // }
    }

    pub(crate) fn end_frame(&mut self, input: &InputState, used_ids: &IdMap<Rect>) {
        // self.caches.update();
        // self.areas.end_frame();
        // self.interaction.focus.end_frame(used_ids);
        // self.drag_value.end_frame(input);
    }
}

/// Some global options that you can read and write.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Controls the tessellator.
    pub tessellation_options: epaint::TessellationOptions,
}
