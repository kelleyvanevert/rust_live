/// What egui emits each frame from [`crate::Context::run`].
///
/// The backend should use this.
#[derive(Clone, Default, PartialEq)]
pub struct FullOutput {
    // /// Non-rendering related output.
    // pub platform_output: PlatformOutput,
    /// If `Duration::is_zero()`, egui is requesting immediate repaint (i.e. on the next frame).
    ///
    /// This happens for instance when there is an animation, or if a user has called `Context::request_repaint()`.
    ///
    /// If `Duration` is greater than zero, egui wants to be repainted at or before the specified
    /// duration elapses. when in reactive mode, egui spends forever waiting for input and only then,
    /// will it repaint itself. this can be used to make sure that backend will only wait for a
    /// specified amount of time, and repaint egui without any new input.
    pub repaint_after: std::time::Duration,
    /// Texture changes since last frame (including the font texture).
    ///
    /// The backend needs to apply [`crate::TexturesDelta::set`] _before_ painting,
    /// and free any texture in [`crate::TexturesDelta::free`] _after_ painting.
    pub textures_delta: epaint::textures::TexturesDelta,

    /// What to paint.
    ///
    /// You can use [`crate::Context::tessellate`] to turn this into triangles.
    pub shapes: Vec<epaint::ClippedShape>,
}
