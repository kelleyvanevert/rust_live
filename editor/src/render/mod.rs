mod buffer;
mod pass;
mod widget_vertex;

use crate::highlight::{syntax_highlight, CodeToken};

use self::{
    buffer::{QuadBufferBuilder, Vertex},
    widget_vertex::{WidgetQuadBufferBuilder, WidgetVertex},
};
use cgmath::SquareMatrix;
use image::GenericImageView;
use live_editor_state::{EditorState, LineSelection, Pos};
use wgpu::util::DeviceExt;
use wgpu_text::{
    glyph_brush::{
        ab_glyph::FontRef, FontId, HorizontalAlign, Layout, OwnedText, Section, Text, VerticalAlign,
    },
    BrushBuilder, TextBrush,
};
use winit::dpi::PhysicalSize;

const CODE_COLOR: [f32; 4] = [0.02, 0.02, 0.02, 1.];
const KW_COLOR: [f32; 4] = [0.02, 0.02, 0.02, 1.];

const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 243.0 / 255.0,
    g: 242.0 / 255.0,
    b: 240.0 / 255.0,
    a: 1.,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SystemUniform {
    view_proj: [[f32; 4]; 4],
}

impl SystemUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update(&mut self, sf: f32, (width, height): (f32, f32)) {
        //             (1,1)
        //        (0,0)
        // (-1,-1)
        let transform = cgmath::Matrix4::from_translation(cgmath::vec3(-1.0, 1.0, 0.0));
        // (0, 0)
        //        (1,1)
        //             (2,2)
        let transform = transform
            * cgmath::Matrix4::from_nonuniform_scale(
                sf * (2.0 / width),
                sf * (2.0 / height) * -1.0,
                1.0,
            );
        // (0,0)
        //      (300,200)
        //              (600,400)

        self.view_proj = transform.into();
    }
}

pub struct Render<'a> {
    pub scale_factor: f32,

    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    // #[allow(dead_code)]
    // adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    render_pipeline: wgpu::RenderPipeline,
    system_uniform: SystemUniform,
    system_bind_group: wgpu::BindGroup,
    system_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    widgets_render_pipeline: wgpu::RenderPipeline,
    widgets_vertex_buffer: wgpu::Buffer,
    widgets_index_buffer: wgpu::Buffer,
    widget_diffuse_bind_group: wgpu::BindGroup,

    // vertex_buffer: wgpu::Buffer,
    // index_buffer: wgpu::Buffer,
    regular_font_id: FontId,
    bold_font_id: FontId,
    code_font_size: f32,
    char_size: (f32, f32),
    title_brush: TextBrush<FontRef<'a>>,
    code_brush: TextBrush<FontRef<'a>>,
    // staging_belt: wgpu::util::StagingBelt,
}

impl<'a> Render<'a> {
    pub async fn new(window: &winit::window::Window) -> Render<'a> {
        let scale_factor = window.scale_factor() as f32;

        // SETUP WGPU STUFF
        // ===

        let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("No adapters found!");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("Surface isn't supported by the adapter.");

        surface.configure(&device, &config);

        // SHADER / BUFFER / TEXTURE STUFF
        // ===

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shader.wgsl").into()),
        });

        // Create pipeline layout
        let system_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 1,
                    //     visibility: wgpu::ShaderStages::FRAGMENT,
                    //     ty: wgpu::BindingType::Texture {
                    //         multisampled: false,
                    //         sample_type: wgpu::TextureSampleType::Uint,
                    //         view_dimension: wgpu::TextureViewDimension::D2,
                    //     },
                    //     count: None,
                    // },
                ],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&system_bind_group_layout],
                push_constant_ranges: &[],
            });

        let mut system_uniform = SystemUniform::new();
        system_uniform.update(scale_factor, (config.width as f32, config.height as f32));

        let system_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("System buffer"),
            contents: bytemuck::cast_slice(&[system_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let system_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &system_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: system_buffer.as_entire_binding(),
                },
                // wgpu::BindGroupEntry {
                //     binding: 1,
                //     resource: wgpu::BindingResource::TextureView(&texture_view),
                // },
            ],
            label: None,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    write_mask: wgpu::ColorWrites::ALL,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: Vertex::SIZE * 400,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: Vertex::SIZE * 400,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // DRAWING SEPARATE IMAGES?
        // TODO: make this dynamic, for widgets
        // ===

        let diffuse_bytes = include_bytes!("../../res/example_waveform.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let dimensions = diffuse_image.dimensions();

        let widget_texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            size: widget_texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB so we need to reflect that here.
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            // This is the same as with the SurfaceConfig. It
            // specifies what texture formats can be used to
            // create TextureViews for this texture. The base
            // texture format (Rgba8UnormSrgb in this case) is
            // always supported. Note that using a different
            // texture format is not supported on the WebGL2
            // backend.
            view_formats: &[],
        });

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            widget_texture_size,
        );

        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let widget_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("widget_texture_bind_group_layout"),
            });

        let widget_diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &widget_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let widgets_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Widgets shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/widgets_shader.wgsl").into()),
        });

        let widgets_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Widgets render pipeline Layout"),
                bind_group_layouts: &[&system_bind_group_layout, &widget_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let widgets_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Widgets render pipeline"),
                layout: Some(&widgets_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &widgets_shader,
                    entry_point: "vs_main", // 1.
                    buffers: &[WidgetVertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &widgets_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: config.format,
                        write_mask: wgpu::ColorWrites::ALL,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });

        let widgets_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Widgets vertex buffer"),
            size: WidgetVertex::SIZE * 400,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let widgets_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Widgets index buffer"),
            size: WidgetVertex::SIZE * 400,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // SETUP TEXT STUFF
        // ===

        let roboto_slab: &[u8] = include_bytes!("../../res/fonts/RobotoSlab-Bold.ttf");

        let title_brush = BrushBuilder::using_font_bytes(roboto_slab).unwrap().build(
            &device,
            config.width,
            config.height,
            config.format,
        );

        let fira_code_bold_font =
            FontRef::try_from_slice(include_bytes!("../../res/fonts/FiraCode-Bold.ttf")).unwrap();

        let fira_code_retina_font =
            FontRef::try_from_slice(include_bytes!("../../res/fonts/FiraCode-Retina.ttf")).unwrap();

        let code_font_size = 50.0;

        let mut code_brush =
            BrushBuilder::using_fonts(vec![fira_code_retina_font.clone(), fira_code_bold_font])
                .build(&device, config.width, config.height, config.format);

        let regular_font_id = FontId(0);
        let bold_font_id = FontId(1);

        let tmp_section = Section::default().add_text(Text::new("x").with_scale(code_font_size));

        let x_bounds = code_brush.glyph_bounds(tmp_section).unwrap();

        let char_size = (x_bounds.width(), x_bounds.height());

        Self {
            scale_factor,

            device,
            queue,
            surface,
            config,

            render_pipeline,
            system_uniform,
            system_bind_group,
            system_buffer,
            vertex_buffer,
            index_buffer,

            widgets_render_pipeline,
            widgets_vertex_buffer,
            widgets_index_buffer,
            widget_diffuse_bind_group,

            regular_font_id,
            bold_font_id,
            code_font_size,
            char_size,
            title_brush,
            code_brush,
        }
    }

    #[allow(unused)]
    pub fn width(&self) -> f32 {
        self.config.width as f32
    }

    #[allow(unused)]
    pub fn height(&self) -> f32 {
        self.config.height as f32
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.config);

        self.title_brush.resize_view(
            self.config.width as f32,
            self.config.height as f32,
            &self.queue,
        );

        self.code_brush.resize_view(
            self.config.width as f32,
            self.config.height as f32,
            &self.queue,
        );

        self.system_uniform.update(
            self.scale_factor,
            (self.config.width as f32, self.config.height as f32),
        );

        self.queue.write_buffer(
            &self.system_buffer,
            0,
            bytemuck::cast_slice(&[self.system_uniform]),
        );
    }

    pub fn pos_to_px(&self, pos: Pos) -> (f32, f32) {
        let sf = self.scale_factor;
        let x = (100.0 + self.char_size.0 * (pos.col as f32)) / sf;
        let y = (260.0 + self.char_size.1 * (pos.row as f32)) / sf;
        (x, y)
    }

    pub fn px_to_pos(&self, (x, y): (f32, f32)) -> Pos {
        let sf = self.scale_factor;
        Pos {
            row: ((y * sf - 260.0) / self.char_size.1).floor() as i32,
            col: ((x * sf - 100.0) / self.char_size.0).round() as i32,
        }
    }

    pub fn render_state(&mut self, editor_state: &EditorState, apply_shader_pipeline: bool) {
        let sf = self.scale_factor;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut builder = QuadBufferBuilder::new();
        let mut widgets_builder = WidgetQuadBufferBuilder::new();

        let title_section = Section::default()
            .add_text(
                Text::new("Some title here")
                    .with_scale(100.0)
                    .with_color([0.01, 0.01, 0.01, 1.0]),
            )
            .with_layout(
                Layout::default()
                    .v_align(VerticalAlign::Top)
                    .h_align(HorizontalAlign::Left),
            )
            // .with_bounds((config.width as f32 - 200.0, config.height as f32))
            .with_screen_position((100.0, 100.0))
            .to_owned();

        let mut code_section = Section::default()
            .with_layout(
                Layout::default()
                    .v_align(VerticalAlign::Top)
                    .h_align(HorizontalAlign::Left),
            )
            .with_screen_position((100.0, 260.0))
            .to_owned();

        let mk_widget_space = |width: usize| {
            OwnedText::new((0..width).map(|_| ' ').collect::<String>())
                .with_font_id(self.bold_font_id)
                .with_scale(self.code_font_size)
                .with_color(KW_COLOR)
        };

        let mk_keyword = |text: String| {
            OwnedText::new(text)
                .with_font_id(self.bold_font_id)
                .with_scale(self.code_font_size)
                .with_color(KW_COLOR)
        };

        let mk_regular = |text: String| {
            OwnedText::new(text)
                .with_font_id(self.regular_font_id)
                .with_scale(self.code_font_size)
                .with_color(CODE_COLOR)
        };

        for (row, line) in syntax_highlight(editor_state.linedata()) {
            for token in line {
                match token {
                    CodeToken::Keyword { text, .. } => code_section.text.push(mk_keyword(text)),
                    CodeToken::Text { text, .. } => code_section.text.push(mk_regular(text)),
                    CodeToken::Widget { col, width, .. } => {
                        code_section.text.push(mk_widget_space(width));

                        let (x_start, y) = self.pos_to_px(Pos {
                            row: row as i32,
                            col: col as i32,
                        });

                        let (x_end, _) = self.pos_to_px(Pos {
                            row: row as i32,
                            col: (col + width) as i32,
                        });

                        widgets_builder.push_quad(
                            x_start,
                            y + 6.0 / sf,
                            x_end,
                            y + self.char_size.1 / sf - 6.0 / sf,
                        );
                    }
                }
            }

            code_section.text.push(mk_regular("\n".into()));
        }

        self.title_brush
            .queue(&self.device, &self.queue, vec![&title_section])
            .unwrap();

        self.code_brush
            .queue(&self.device, &self.queue, vec![&code_section])
            .unwrap();

        for LineSelection {
            row,
            col_start,
            col_end,
        } in editor_state.visual_selections()
        {
            let (x_start, y) = self.pos_to_px(Pos {
                row,
                col: col_start,
            });

            let (x_end, _) = self.pos_to_px(Pos { row, col: col_end });

            builder.push_quad(
                x_start,
                y,
                x_end + 6.0 / sf,
                y + self.char_size.1 / sf,
                [0.0, 0.0, 0.0, 0.2],
            );
        }

        for caret in editor_state.caret_positions() {
            let (cx, cy) = self.pos_to_px(caret);

            builder.push_quad(
                cx,
                cy,
                cx + 6.0 / sf,
                cy + self.char_size.1 / sf,
                [0.0, 0.0, 0.0, 1.0],
            );
        }

        let num_indices = {
            let (stg_vertex, stg_index, num_indices) = builder.build(&self.device);

            stg_vertex.copy_to_buffer(&mut encoder, &self.vertex_buffer);
            stg_index.copy_to_buffer(&mut encoder, &self.index_buffer);

            num_indices
        };

        let widgets_num_indices = {
            let (stg_vertex, stg_index, widgets_num_indices) = widgets_builder.build(&self.device);

            stg_vertex.copy_to_buffer(&mut encoder, &self.widgets_vertex_buffer);
            stg_index.copy_to_buffer(&mut encoder, &self.widgets_index_buffer);

            widgets_num_indices
        };

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture!");

        let view = frame.texture.create_view(&Default::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,

                    // 1. Clear background
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // 2. Draw text
            self.title_brush.draw(&mut render_pass);
            self.code_brush.draw(&mut render_pass);

            // 3. Draw widgets
            if apply_shader_pipeline {
                render_pass.set_pipeline(&self.widgets_render_pipeline);
                render_pass.set_bind_group(0, &self.system_bind_group, &[]);
                render_pass.set_bind_group(1, &self.widget_diffuse_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.widgets_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.widgets_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                ); // 1.
                render_pass.draw_indexed(0..widgets_num_indices, 0, 0..1); // 2.
            }

            // 4. Draw selections and carets
            if apply_shader_pipeline {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.system_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32); // 1.
                render_pass.draw_indexed(0..num_indices, 0, 0..1); // 2.
            }
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
    }
}
