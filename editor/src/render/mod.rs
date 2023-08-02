mod buffer;
mod pass;
mod system;
mod widget_vertex;
mod widgets_pass;

use crate::highlight::{syntax_highlight, CodeToken};

use self::{
    buffer::{QuadBufferBuilder, Vertex},
    system::SystemData,
    widgets_pass::WidgetsPass,
};
use live_editor_state::{EditorState, LineSelection, Pos};
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

pub struct Render<'a> {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    // #[allow(dead_code)]
    // adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    pub system: SystemData,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    widgets_pass: WidgetsPass,

    regular_font_id: FontId,
    bold_font_id: FontId,
    code_font_size: f32,
    char_size: (f32, f32),
    title_brush: TextBrush<FontRef<'a>>,
    code_brush: TextBrush<FontRef<'a>>,
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

        let system = SystemData::new(scale_factor, char_size, &device, &queue, &config);

        // Create pipeline layout

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&system.bind_group_layout],
                push_constant_ranges: &[],
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

        let widgets_pass = WidgetsPass::new(&device, &queue, &config, &system);

        Self {
            device,
            queue,
            surface,
            config,

            render_pipeline,
            system,
            vertex_buffer,
            index_buffer,

            widgets_pass,

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

    pub fn resize(&mut self, mut size: PhysicalSize<u32>) {
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

        self.system.resize(&self.queue, &self.config);
    }

    pub fn render_state(&mut self, editor_state: &EditorState, apply_shader_pipeline: bool) {
        let sf = self.system.scale_factor;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut builder = QuadBufferBuilder::new();

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

        let mk_keyword = |text: &str| {
            OwnedText::new(text)
                .with_font_id(self.bold_font_id)
                .with_scale(self.code_font_size)
                .with_color(KW_COLOR)
        };

        let mk_regular = |text: &str| {
            OwnedText::new(text)
                .with_font_id(self.regular_font_id)
                .with_scale(self.code_font_size)
                .with_color(CODE_COLOR)
        };

        let code = syntax_highlight(editor_state.linedata());

        for (row, line) in &code {
            for token in line {
                match token {
                    CodeToken::Keyword { text, .. } => code_section.text.push(mk_keyword(text)),
                    CodeToken::Text { text, .. } => code_section.text.push(mk_regular(text)),
                    CodeToken::Widget { col, width, .. } => {
                        code_section.text.push(mk_widget_space(*width));

                        let (x_start, y) = self.system.pos_to_px(Pos {
                            row: *row as i32,
                            col: *col as i32,
                        });

                        let (x_end, _) = self.system.pos_to_px(Pos {
                            row: *row as i32,
                            col: (col + width) as i32,
                        });
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
            let (x_start, y) = self.system.pos_to_px(Pos {
                row,
                col: col_start,
            });

            let (x_end, _) = self.system.pos_to_px(Pos { row, col: col_end });

            builder.push_quad(
                x_start,
                y,
                x_end + 6.0 / sf,
                y + self.char_size.1 / sf,
                [0.0, 0.0, 0.0, 0.2],
            );
        }

        for caret in editor_state.caret_positions() {
            let (cx, cy) = self.system.pos_to_px(caret);

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

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture!");

        let view = frame.texture.create_view(&Default::default());

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,

                    // 1. Clear background
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // 2. Draw text
            self.title_brush.draw(&mut render_pass);
            self.code_brush.draw(&mut render_pass);
        }

        self.widgets_pass.render_state(
            &self.device,
            &self.system,
            // self.scale_factor,
            // self.char_size,
            // |pos: Pos| self.pos_to_px(pos),
            // |px: (f32, f32)| self.px_to_pos(px),
            &view,
            &code,
            &mut encoder,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,

                    // 1. Clear background
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // 4. Draw selections and carets
            if apply_shader_pipeline {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.system.bind_group, &[]);
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
