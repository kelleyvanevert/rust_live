use emath::Rect;

use super::{context::InputState, id::IdMap};

/// State that is collected during a frame and then cleared.
/// Short-term (single frame) memory.
#[derive(Clone)]
pub(crate) struct FrameState {
    /// All [`Id`]s that were used this frame.
    pub(crate) used_ids: IdMap<Rect>,
    // /// Starts off as the screen_rect, shrinks as panels are added.
    // /// The [`CentralPanel`] does not change this.
    // /// This is the area available to Window's.
    // pub(crate) available_rect: Rect,

    // /// Starts off as the screen_rect, shrinks as panels are added.
    // /// The [`CentralPanel`] retracts from this.
    // pub(crate) unused_rect: Rect,

    // /// How much space is used by panels.
    // pub(crate) used_by_panels: Rect,

    // /// If a tooltip has been shown this frame, where was it?
    // /// This is used to prevent multiple tooltips to cover each other.
    // /// Initialized to `None` at the start of each frame.
    // pub(crate) tooltip_state: Option<TooltipFrameState>,

    // /// Set to [`InputState::scroll_delta`] on the start of each frame.
    // ///
    // /// Cleared by the first [`ScrollArea`] that makes use of it.
    // pub(crate) scroll_delta: Vec2, // TODO(emilk): move to `InputState` ?

    // /// horizontal, vertical
    // pub(crate) scroll_target: [Option<(Rangef, Option<Align>)>; 2],

    // #[cfg(feature = "accesskit")]
    // pub(crate) accesskit_state: Option<AccessKitFrameState>,

    // /// Highlight these widgets this next frame. Read from this.
    // pub(crate) highlight_this_frame: IdSet,

    // /// Highlight these widgets the next frame. Write to this.
    // pub(crate) highlight_next_frame: IdSet,
}

impl FrameState {
    pub(crate) fn begin_frame(&mut self, input: &InputState) {
        let Self {
            used_ids,
            // available_rect,
            // unused_rect,
            // used_by_panels,
            // tooltip_state,
            // scroll_delta,
            // scroll_target,
            // #[cfg(feature = "accesskit")]
            // accesskit_state,
            // highlight_this_frame,
            // highlight_next_frame,
        } = self;

        used_ids.clear();
        // *available_rect = input.screen_rect();
        // *unused_rect = input.screen_rect();
        // *used_by_panels = Rect::NOTHING;
        // *tooltip_state = None;
        // *scroll_delta = input.scroll_delta;
        // *scroll_target = [None, None];

        // #[cfg(feature = "accesskit")]
        // {
        //     *accesskit_state = None;
        // }

        // *highlight_this_frame = std::mem::take(highlight_next_frame);
    }
}

impl Default for FrameState {
    fn default() -> Self {
        Self {
            used_ids: Default::default(),
            // available_rect: Rect::NAN,
            // unused_rect: Rect::NAN,
            // used_by_panels: Rect::NAN,
            // tooltip_state: None,
            // scroll_delta: Vec2::ZERO,
            // scroll_target: [None, None],
            // #[cfg(feature = "accesskit")]
            // accesskit_state: None,
            // highlight_this_frame: Default::default(),
            // highlight_next_frame: Default::default(),
        }
    }
}
