use std::sync::Arc;

use emath::{vec2, Rect};
use epaint::{
    mutex::RwLock, tessellator, text::FontDefinitions, ClippedPrimitive, ClippedShape, Fonts,
    PaintStats, TextureId,
};

use super::{
    frame_state::FrameState,
    input::RawInput,
    layers::GraphicLayers,
    memory::{Memory, Options},
    output::FullOutput,
};

#[derive(Clone, Debug)]
pub struct InputState {
    /// Also known as device pixel ratio, > 1 for high resolution screens.
    pub pixels_per_point: f32,

    /// Position and size of the egui area.
    pub screen_rect: Rect,

    /// Maximum size of one side of a texture.
    ///
    /// This depends on the backend.
    pub max_texture_side: usize,
}

impl InputState {
    /// Also known as device pixel ratio, > 1 for high resolution screens.
    #[inline(always)]
    pub fn pixels_per_point(&self) -> f32 {
        self.pixels_per_point
    }

    /// Size of a physical pixel in logical gui coordinates (points).
    #[inline(always)]
    pub fn physical_pixel_size(&self) -> f32 {
        1.0 / self.pixels_per_point()
    }

    #[must_use]
    pub fn begin_frame(
        mut self,
        mut new: RawInput,
        requested_repaint_last_frame: bool,
    ) -> InputState {
        // let time = new.time.unwrap_or(self.time + new.predicted_dt as f64);
        // let unstable_dt = (time - self.time) as f32;

        // let stable_dt = if requested_repaint_last_frame {
        //     // we should have had a repaint straight away,
        //     // so this should be trustable.
        //     unstable_dt
        // } else {
        //     new.predicted_dt
        // };

        let screen_rect = new.screen_rect.unwrap_or(self.screen_rect);
        // self.create_touch_states_for_new_devices(&new.events);
        // for touch_state in self.touch_states.values_mut() {
        //     touch_state.begin_frame(time, &new, self.pointer.interact_pos);
        // }
        // let pointer = self.pointer.begin_frame(time, &new);

        // let mut keys_down = self.keys_down;
        // let mut scroll_delta = Vec2::ZERO;
        // let mut zoom_factor_delta = 1.0;
        // for event in &mut new.events {
        //     match event {
        //         Event::Key {
        //             key,
        //             pressed,
        //             repeat,
        //             ..
        //         } => {
        //             if *pressed {
        //                 let first_press = keys_down.insert(*key);
        //                 *repeat = !first_press;
        //             } else {
        //                 keys_down.remove(key);
        //             }
        //         }
        //         Event::Scroll(delta) => {
        //             scroll_delta += *delta;
        //         }
        //         Event::Zoom(factor) => {
        //             zoom_factor_delta *= *factor;
        //         }
        //         _ => {}
        //     }
        // }

        // let mut modifiers = new.modifiers;

        // let focused_changed = self.focused != new.focused
        //     || new
        //         .events
        //         .iter()
        //         .any(|e| matches!(e, Event::WindowFocused(_)));
        // if focused_changed {
        //     // It is very common for keys to become stuck when we alt-tab, or a save-dialog opens by Ctrl+S.
        //     // Therefore we clear all the modifiers and down keys here to avoid that.
        //     modifiers = Default::default();
        //     keys_down = Default::default();
        // }

        InputState {
            // pointer,
            // touch_states: self.touch_states,
            // scroll_delta,
            // zoom_factor_delta,
            screen_rect,
            pixels_per_point: new.pixels_per_point.unwrap_or(self.pixels_per_point),
            max_texture_side: new.max_texture_side.unwrap_or(self.max_texture_side),
            // time,
            // unstable_dt,
            // predicted_dt: new.predicted_dt,
            // stable_dt,
            // focused: new.focused,
            // modifiers,
            // keys_down,
            // events: new.events.clone(), // TODO(emilk): remove clone() and use raw.events
            // raw: new,
        }
    }

    #[inline(always)]
    pub fn screen_rect(&self) -> Rect {
        self.screen_rect
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            // raw: Default::default(),
            // pointer: Default::default(),
            // touch_states: Default::default(),
            // scroll_delta: Vec2::ZERO,
            // zoom_factor_delta: 1.0,
            screen_rect: Rect::from_min_size(Default::default(), vec2(10_000.0, 10_000.0)),
            pixels_per_point: 1.0,
            max_texture_side: 2048,
            // time: 0.0,
            // unstable_dt: 1.0 / 60.0,
            // predicted_dt: 1.0 / 60.0,
            // stable_dt: 1.0 / 60.0,
            // focused: false,
            // modifiers: Default::default(),
            // keys_down: Default::default(),
            // events: Default::default(),
        }
    }
}

struct WrappedTextureManager(Arc<RwLock<epaint::TextureManager>>);

impl Default for WrappedTextureManager {
    fn default() -> Self {
        let mut tex_mngr = epaint::textures::TextureManager::default();

        // Will be filled in later
        let font_id = tex_mngr.alloc(
            "egui_font_texture".into(),
            epaint::FontImage::new([0, 0]).into(),
            Default::default(),
        );
        assert_eq!(font_id, TextureId::default());

        Self(Arc::new(RwLock::new(tex_mngr)))
    }
}

#[derive(Clone)]
pub struct KguiContext(Arc<RwLock<KguiContextImpl>>);

impl KguiContext {
    // Do read-only (shared access) transaction on Context
    fn read<R>(&self, reader: impl FnOnce(&KguiContextImpl) -> R) -> R {
        reader(&self.0.read())
    }

    // Do read-write (exclusive access) transaction on Context
    fn write<R>(&self, writer: impl FnOnce(&mut KguiContextImpl) -> R) -> R {
        writer(&mut self.0.write())
    }

    /// Tell `egui` which fonts to use.
    ///
    /// The default `egui` fonts only support latin and cyrillic alphabets,
    /// but you can call this to install additional fonts that support e.g. korean characters.
    ///
    /// The new fonts will become active at the start of the next frame.
    pub fn set_fonts(&self, font_definitions: FontDefinitions) {
        let update_fonts = self.fonts_mut(|fonts| {
            if let Some(current_fonts) = fonts {
                // NOTE: this comparison is expensive since it checks TTF data for equality
                current_fonts.lock().fonts.definitions() != &font_definitions
            } else {
                true
            }
        });

        if update_fonts {
            self.memory_mut(|mem| mem.new_font_definitions = Some(font_definitions));
        }
    }

    /// Read-only access to [`Fonts`].
    ///
    /// Not valid until first call to [`Context::run()`].
    /// That's because since we don't know the proper `pixels_per_point` until then.
    #[inline]
    pub fn fonts<R>(&self, reader: impl FnOnce(&Fonts) -> R) -> R {
        self.read(move |ctx| {
            reader(
                ctx.fonts
                    .as_ref()
                    .expect("No fonts available until first call to Context::run()"),
            )
        })
    }

    /// Read-only access to [`Memory`].
    #[inline]
    pub fn memory<R>(&self, reader: impl FnOnce(&Memory) -> R) -> R {
        self.read(move |ctx| reader(&ctx.memory))
    }

    /// Read-write access to [`Memory`].
    #[inline]
    pub fn memory_mut<R>(&self, writer: impl FnOnce(&mut Memory) -> R) -> R {
        self.write(move |ctx| writer(&mut ctx.memory))
    }

    /// Read-write access to [`Fonts`].
    #[inline]
    pub fn fonts_mut<R>(&self, writer: impl FnOnce(&mut Option<Fonts>) -> R) -> R {
        self.write(move |ctx| writer(&mut ctx.fonts))
    }

    /// Read-only access to [`Options`].
    #[inline]
    pub fn options<R>(&self, reader: impl FnOnce(&Options) -> R) -> R {
        self.read(move |ctx| reader(&ctx.memory.options))
    }

    /// Read-write access to [`Options`].
    #[inline]
    pub fn options_mut<R>(&self, writer: impl FnOnce(&mut Options) -> R) -> R {
        self.write(move |ctx| writer(&mut ctx.memory.options))
    }

    // /// Read-only access to [`IdTypeMap`], which stores superficial widget state.
    // #[inline]
    // pub fn data<R>(&self, reader: impl FnOnce(&IdTypeMap) -> R) -> R {
    //     self.read(move |ctx| reader(&ctx.memory.data))
    // }

    // /// Read-write access to [`IdTypeMap`], which stores superficial widget state.
    // #[inline]
    // pub fn data_mut<R>(&self, writer: impl FnOnce(&mut IdTypeMap) -> R) -> R {
    //     self.write(move |ctx| writer(&mut ctx.memory.data))
    // }

    /// Read-write access to [`GraphicLayers`], where painted [`crate::Shape`]s are written to.
    #[inline]
    pub(crate) fn graphics_mut<R>(&self, writer: impl FnOnce(&mut GraphicLayers) -> R) -> R {
        self.write(move |ctx| writer(&mut ctx.graphics))
    }

    /// Call at the end of each frame.
    #[must_use]
    pub fn end_frame(&self) -> FullOutput {
        // if self.input(|i| i.wants_repaint()) {
        //     self.request_repaint();
        // }

        let textures_delta = self.write(|ctx| {
            ctx.memory.end_frame(&ctx.input, &ctx.frame_state.used_ids);

            let font_image_delta = ctx.fonts.as_ref().unwrap().font_image_delta();
            if let Some(font_image_delta) = font_image_delta {
                ctx.tex_manager
                    .0
                    .write()
                    .set(TextureId::default(), font_image_delta);
            }

            ctx.tex_manager.0.write().take_delta()
        });

        // #[cfg_attr(not(feature = "accesskit"), allow(unused_mut))]
        // let mut platform_output: PlatformOutput = self.output_mut(|o| std::mem::take(o));

        // #[cfg(feature = "accesskit")]
        // {
        //     let state = self.frame_state_mut(|fs| fs.accesskit_state.take());
        //     if let Some(state) = state {
        //         let has_focus = self.input(|i| i.raw.focused);
        //         let root_id = crate::accesskit_root_id().accesskit_id();
        //         let nodes = self.write(|ctx| {
        //             state
        //                 .node_builders
        //                 .into_iter()
        //                 .map(|(id, builder)| {
        //                     (
        //                         id.accesskit_id(),
        //                         builder.build(&mut ctx.accesskit_node_classes),
        //                     )
        //                 })
        //                 .collect()
        //         });
        //         platform_output.accesskit_update = Some(accesskit::TreeUpdate {
        //             nodes,
        //             tree: Some(accesskit::Tree::new(root_id)),
        //             focus: has_focus.then(|| {
        //                 let focus_id = self.memory(|mem| mem.interaction.focus.id);
        //                 focus_id.map_or(root_id, |id| id.accesskit_id())
        //             }),
        //         });
        //     }
        // }

        let repaint_after = self.write(|ctx| ctx.repaint.end_frame());
        let shapes = self.drain_paint_lists();

        FullOutput {
            // platform_output,
            repaint_after,
            textures_delta,
            shapes,
        }
    }

    fn drain_paint_lists(&self) -> Vec<ClippedShape> {
        self.write(|ctx| ctx.graphics.drain().collect())
    }

    /// Tessellate the given shapes into triangle meshes.
    pub fn tessellate(&self, shapes: Vec<ClippedShape>) -> Vec<ClippedPrimitive> {
        // A tempting optimization is to reuse the tessellation from last frame if the
        // shapes are the same, but just comparing the shapes takes about 50% of the time
        // it takes to tessellate them, so it is not a worth optimization.

        // here we expect that we are the only user of context, since frame is ended
        self.write(|ctx| {
            let pixels_per_point = ctx.input.pixels_per_point();
            let tessellation_options = ctx.memory.options.tessellation_options;
            let texture_atlas = ctx
                .fonts
                .as_ref()
                .expect("tessellate called before first call to Context::run()")
                .texture_atlas();
            let (font_tex_size, prepared_discs) = {
                let atlas = texture_atlas.lock();
                (atlas.size(), atlas.prepared_discs())
            };

            let paint_stats = PaintStats::from_shapes(&shapes);
            let clipped_primitives = tessellator::tessellate_shapes(
                pixels_per_point,
                tessellation_options,
                font_tex_size,
                prepared_discs,
                shapes,
            );
            ctx.paint_stats = paint_stats.with_clipped_primitives(&clipped_primitives);
            clipped_primitives
        })
    }

    /// An alternative to calling [`Self::run`].
    ///
    /// ```
    /// // One egui context that you keep reusing:
    /// let mut ctx = egui::Context::default();
    ///
    /// // Each frame:
    /// let input = egui::RawInput::default();
    /// ctx.begin_frame(input);
    ///
    /// egui::CentralPanel::default().show(&ctx, |ui| {
    ///     ui.label("Hello egui!");
    /// });
    ///
    /// let full_output = ctx.end_frame();
    /// // handle full_output
    /// ```
    pub fn begin_frame(&self, new_input: RawInput) {
        self.write(|ctx| ctx.begin_frame_mut(new_input));
    }
}

impl Default for KguiContext {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(KguiContextImpl::default())))
    }
}

#[derive(Default)]
struct KguiContextImpl {
    /// `None` until the start of the first frame.
    fonts: Option<Fonts>,
    memory: Memory,
    // animation_manager: AnimationManager,
    tex_manager: WrappedTextureManager,
    //
    // os: OperatingSystem,
    //
    input: InputState,
    //
    /// State that is collected during a frame and then cleared
    frame_state: FrameState,
    //
    // The output of a frame:
    graphics: GraphicLayers,
    // output: PlatformOutput,
    //
    paint_stats: PaintStats,

    repaint: Repaint,
    //
    /// Written to during the frame.
    // layer_rects_this_frame: ahash::HashMap<LayerId, Vec<(Id, Rect)>>,
    //
    /// Read
    // layer_rects_prev_frame: ahash::HashMap<LayerId, Vec<(Id, Rect)>>,
    //
    #[cfg(feature = "accesskit")]
    is_accesskit_enabled: bool,
    #[cfg(feature = "accesskit")]
    accesskit_node_classes: accesskit::NodeClassSet,
}

impl KguiContextImpl {
    fn begin_frame_mut(&mut self, mut new_raw_input: RawInput) {
        self.repaint.start_frame();

        if let Some(new_pixels_per_point) = self.memory.new_pixels_per_point.take() {
            new_raw_input.pixels_per_point = Some(new_pixels_per_point);

            // This is a bit hacky, but is required to avoid jitter:
            let ratio = self.input.pixels_per_point / new_pixels_per_point;
            let mut rect = self.input.screen_rect;
            rect.min = (ratio * rect.min.to_vec2()).to_pos2();
            rect.max = (ratio * rect.max.to_vec2()).to_pos2();
            new_raw_input.screen_rect = Some(rect);
        }

        // self.layer_rects_prev_frame = std::mem::take(&mut self.layer_rects_this_frame);

        self.memory.begin_frame(&self.input, &new_raw_input);

        self.input = std::mem::take(&mut self.input)
            .begin_frame(new_raw_input, self.repaint.requested_repaint_last_frame);

        self.frame_state.begin_frame(&self.input);

        self.update_fonts_mut();

        // Ensure we register the background area so panels and background ui can catch clicks:
        let screen_rect = self.input.screen_rect();
        // self.memory.areas.set_state(
        //     LayerId::background(),
        //     containers::area::State {
        //         pivot_pos: screen_rect.left_top(),
        //         pivot: Align2::LEFT_TOP,
        //         size: screen_rect.size(),
        //         interactable: true,
        //     },
        // );

        #[cfg(feature = "accesskit")]
        if self.is_accesskit_enabled {
            use crate::frame_state::AccessKitFrameState;
            let id = crate::accesskit_root_id();
            let mut builder = accesskit::NodeBuilder::new(accesskit::Role::Window);
            builder.set_transform(accesskit::Affine::scale(
                self.input.pixels_per_point().into(),
            ));
            let mut node_builders = IdMap::default();
            node_builders.insert(id, builder);
            self.frame_state.accesskit_state = Some(AccessKitFrameState {
                node_builders,
                parent_stack: vec![id],
            });
        }
    }

    /// Load fonts unless already loaded.
    fn update_fonts_mut(&mut self) {
        let pixels_per_point = self.input.pixels_per_point();
        let max_texture_side = self.input.max_texture_side;

        if let Some(font_definitions) = self.memory.new_font_definitions.take() {
            let fonts = Fonts::new(pixels_per_point, max_texture_side, font_definitions);
            self.fonts = Some(fonts);
        }

        let fonts = self.fonts.get_or_insert_with(|| {
            let font_definitions = FontDefinitions::default();
            Fonts::new(pixels_per_point, max_texture_side, font_definitions)
        });

        fonts.begin_frame(pixels_per_point, max_texture_side);

        // if self.memory.options.preload_font_glyphs {
        //     // Preload the most common characters for the most common fonts.
        //     // This is not very important to do, but may a few GPU operations.
        //     for font_id in self.memory.options.style.text_styles.values() {
        //         fonts.lock().fonts.font(font_id).preload_common_characters();
        //     }
        // }
    }

    #[cfg(feature = "accesskit")]
    fn accesskit_node_builder(&mut self, id: Id) -> &mut accesskit::NodeBuilder {
        let state = self.frame_state.accesskit_state.as_mut().unwrap();
        let builders = &mut state.node_builders;
        if let std::collections::hash_map::Entry::Vacant(entry) = builders.entry(id) {
            entry.insert(Default::default());
            let parent_id = state.parent_stack.last().unwrap();
            let parent_builder = builders.get_mut(parent_id).unwrap();
            parent_builder.push_child(id.accesskit_id());
        }
        builders.get_mut(&id).unwrap()
    }
}

/// Logic related to repainting the ui.
struct Repaint {
    /// The current frame number.
    ///
    /// Incremented at the end of each frame.
    frame_nr: u64,

    /// The duration backend will poll for new events, before forcing another egui update
    /// even if there's no new events.
    ///
    /// Also used to suppress multiple calls to the repaint callback during the same frame.
    repaint_after: std::time::Duration,

    /// While positive, keep requesting repaints. Decrement at the end of each frame.
    repaint_requests: u32,
    request_repaint_callback: Option<Box<dyn Fn(RequestRepaintInfo) + Send + Sync>>,

    requested_repaint_last_frame: bool,
}

impl Default for Repaint {
    fn default() -> Self {
        Self {
            frame_nr: 0,
            repaint_after: std::time::Duration::from_millis(100),
            // Start with painting an extra frame to compensate for some widgets
            // that take two frames before they "settle":
            repaint_requests: 1,
            request_repaint_callback: None,
            requested_repaint_last_frame: false,
        }
    }
}

impl Repaint {
    fn request_repaint(&mut self) {
        self.request_repaint_after(std::time::Duration::ZERO);
    }

    fn request_repaint_after(&mut self, after: std::time::Duration) {
        if after == std::time::Duration::ZERO {
            // Do a few extra frames to let things settle.
            // This is a bit of a hack, and we don't support it for `repaint_after` callbacks yet.
            self.repaint_requests = 2;
        }

        // We only re-call the callback if we get a lower duration,
        // otherwise it's already been covered by the previous callback.
        if after < self.repaint_after {
            self.repaint_after = after;

            if let Some(callback) = &self.request_repaint_callback {
                let info = RequestRepaintInfo {
                    after,
                    current_frame_nr: self.frame_nr,
                };
                (callback)(info);
            }
        }
    }

    fn start_frame(&mut self) {
        // We are repainting; no need to reschedule a repaint unless the user asks for it again.
        self.repaint_after = std::time::Duration::MAX;
    }

    // returns how long to wait until repaint
    fn end_frame(&mut self) -> std::time::Duration {
        // if repaint_requests is greater than zero. just set the duration to zero for immediate
        // repaint. if there's no repaint requests, then we can use the actual repaint_after instead.
        let repaint_after = if self.repaint_requests > 0 {
            self.repaint_requests -= 1;
            std::time::Duration::ZERO
        } else {
            self.repaint_after
        };
        self.repaint_after = std::time::Duration::MAX;

        self.requested_repaint_last_frame = repaint_after.is_zero();
        self.frame_nr += 1;

        repaint_after
    }
}

/// Information given to the backend about when it is time to repaint the ui.
///
/// This is given in the callback set by [`Context::set_request_repaint_callback`].
#[derive(Clone, Copy, Debug)]
pub struct RequestRepaintInfo {
    /// Repaint after this duration. If zero, repaint as soon as possible.
    pub after: std::time::Duration,

    /// The current frame number.
    ///
    /// This can be compared to [`Context::frame_nr`] to see if we've already
    /// triggered the painting of the next frame.
    pub current_frame_nr: u64,
}
