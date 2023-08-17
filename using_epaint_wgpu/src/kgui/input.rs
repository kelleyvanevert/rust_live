use emath::Rect;

/// What the integrations provides to egui at the start of each frame.
///
/// Set the values that make sense, leave the rest at their `Default::default()`.
///
/// You can check if `egui` is using the inputs using
/// [`crate::Context::wants_pointer_input`] and [`crate::Context::wants_keyboard_input`].
///
/// All coordinates are in points (logical pixels) with origin (0, 0) in the top left corner.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RawInput {
    /// Position and size of the area that egui should use, in points.
    /// Usually you would set this to
    ///
    /// `Some(Rect::from_min_size(Default::default(), screen_size_in_points))`.
    ///
    /// but you could also constrain egui to some smaller portion of your window if you like.
    ///
    /// `None` will be treated as "same as last frame", with the default being a very big area.
    pub screen_rect: Option<Rect>,

    /// Also known as device pixel ratio, > 1 for high resolution screens.
    /// If text looks blurry you probably forgot to set this.
    /// Set this the first frame, whenever it changes, or just on every frame.
    pub pixels_per_point: Option<f32>,
    /// Maximum size of one side of the font texture.
    ///
    /// Ask your graphics drivers about this. This corresponds to `GL_MAX_TEXTURE_SIZE`.
    ///
    /// The default is a very small (but very portable) 2048.
    pub max_texture_side: Option<usize>,
    //
    // /// Monotonically increasing time, in seconds. Relative to whatever. Used for animations.
    // /// If `None` is provided, egui will assume a time delta of `predicted_dt` (default 1/60 seconds).
    // pub time: Option<f64>,

    // /// Should be set to the expected time between frames when painting at vsync speeds.
    // /// The default for this is 1/60.
    // /// Can safely be left at its default value.
    // pub predicted_dt: f32,

    // /// Which modifier keys are down at the start of the frame?
    // pub modifiers: Modifiers,

    // /// In-order events received this frame.
    // ///
    // /// There is currently no way to know if egui handles a particular event,
    // /// but you can check if egui is using the keyboard with [`crate::Context::wants_keyboard_input`]
    // /// and/or the pointer (mouse/touch) with [`crate::Context::is_using_pointer`].
    // pub events: Vec<Event>,

    // /// Dragged files hovering over egui.
    // pub hovered_files: Vec<HoveredFile>,

    // /// Dragged files dropped into egui.
    // ///
    // /// Note: when using `eframe` on Windows you need to enable
    // /// drag-and-drop support using `eframe::NativeOptions`.
    // pub dropped_files: Vec<DroppedFile>,

    // /// The native window has the keyboard focus (i.e. is receiving key presses).
    // ///
    // /// False when the user alt-tab away from the application, for instance.
    // pub focused: bool,
}

impl RawInput {
    /// Helper: move volatile (deltas and events), clone the rest.
    ///
    /// * [`Self::hovered_files`] is cloned.
    /// * [`Self::dropped_files`] is moved.
    pub fn take(&mut self) -> RawInput {
        RawInput {
            screen_rect: self.screen_rect.take(),
            pixels_per_point: self.pixels_per_point.take(),
            max_texture_side: self.max_texture_side.take(),
            // time: self.time.take(),
            // predicted_dt: self.predicted_dt,
            // modifiers: self.modifiers,
            // events: std::mem::take(&mut self.events),
            // hovered_files: self.hovered_files.clone(),
            // dropped_files: std::mem::take(&mut self.dropped_files),
            // focused: self.focused,
        }
    }
}

impl Default for RawInput {
    fn default() -> Self {
        Self {
            screen_rect: None,
            pixels_per_point: None,
            max_texture_side: None,
            // time: None,
            // predicted_dt: 1.0 / 60.0,
            // modifiers: Modifiers::default(),
            // events: vec![],
            // hovered_files: Default::default(),
            // dropped_files: Default::default(),
            // focused: true, // integrations opt into global focus tracking
        }
    }
}
