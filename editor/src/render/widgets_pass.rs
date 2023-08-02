use crate::highlight::CodeToken;

use super::widget_vertex::{WidgetQuadBufferBuilder, WidgetVertex};

use image::GenericImageView;
use live_editor_state::Pos;
use wgpu::TextureView;

pub struct WidgetsPass {
    scale_factor: f32,
    char_size: (f32, f32),

    widgets_render_pipeline: wgpu::RenderPipeline,
    widgets_vertex_buffer: wgpu::Buffer,
    widgets_index_buffer: wgpu::Buffer,
    widget_diffuse_bind_group: wgpu::BindGroup,
}

impl WidgetsPass {
    pub fn new(
        scale_factor: f32,
        char_size: (f32, f32),
        // phong_config: &PhongConfig,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        system_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
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

        Self {
            scale_factor,
            char_size,
            widgets_render_pipeline,
            widgets_vertex_buffer,
            widgets_index_buffer,
            widget_diffuse_bind_group,
        }
    }

    fn draw(
        &mut self,
        // surface: &wgpu::Surface,
        // device: &wgpu::Device,
        // queue: &wgpu::Queue,
        // system_bind_group: &wgpu::BindGroup,
        // obj_model: &Model,
    ) -> Result<(), wgpu::SurfaceError> {
        todo!()
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

    pub fn render_state(
        &mut self,
        device: &wgpu::Device,
        system_bind_group: &wgpu::BindGroup,
        // render: &Render,
        // pos_to_px: impl Fn(Pos) -> (f32, f32),
        // px_to_pos: impl Fn((f32, f32)) -> Pos,
        view: &TextureView,
        code: &[(usize, Vec<CodeToken>)],
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let sf = self.scale_factor;

        let mut widgets_builder = WidgetQuadBufferBuilder::new();

        for (row, line) in code {
            for token in line {
                match token {
                    CodeToken::Widget { col, width, .. } => {
                        let (x_start, y) = self.pos_to_px(Pos {
                            row: *row as i32,
                            col: *col as i32,
                        });

                        let (x_end, _) = self.pos_to_px(Pos {
                            row: *row as i32,
                            col: (col + width) as i32,
                        });

                        widgets_builder.push_quad(
                            x_start,
                            y + 6.0 / sf,
                            x_end,
                            y + self.char_size.1 / sf - 6.0 / sf,
                        );
                    }
                    _ => {}
                }
            }
        }

        let (stg_vertex, stg_index, widgets_num_indices) = widgets_builder.build(&device);

        stg_vertex.copy_to_buffer(encoder, &self.widgets_vertex_buffer);
        stg_index.copy_to_buffer(encoder, &self.widgets_index_buffer);

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

        render_pass.set_pipeline(&self.widgets_render_pipeline);
        render_pass.set_bind_group(0, &system_bind_group, &[]);
        render_pass.set_bind_group(1, &self.widget_diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.widgets_vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.widgets_index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        ); // 1.
        render_pass.draw_indexed(0..widgets_num_indices, 0, 0..1); // 2.
    }
}

// impl Pass for WidgetsPass {
// }
