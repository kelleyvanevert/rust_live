use std::collections::HashMap;

use crate::widget::WidgetManager;

use super::{
    system::SystemData,
    widget_vertex::{WidgetQuadBufferBuilder, WidgetVertex},
};

use wgpu::TextureView;

pub struct WidgetsPass {
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    widget_textures: HashMap<usize, WidgetTexture>,
}

impl WidgetsPass {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        system: &SystemData,
    ) -> Self {
        let texture_bind_group_layout =
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
                label: Some("texture_bind_group_layout"),
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Widgets shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/widgets_shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Widgets render pipeline Layout"),
                bind_group_layouts: &[&system.bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Widgets render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[WidgetVertex::desc()],
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

        Self {
            render_pipeline,
            texture_bind_group_layout,
            widget_textures: HashMap::new(),
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        system: &SystemData,
        view: &TextureView,
        widget_instances: Vec<(usize, (f32, f32, f32, f32))>,
        widget_manager: &mut WidgetManager,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        for group in widget_instances.group_by(|a, b| a.0 == b.0) {
            let id = group[0].0;
            let (min_x, min_y, max_x, max_y) = group[0].1;
            let width = (max_x - min_x).round() as usize;
            let height = (max_y - min_y).round() as usize;

            let widget_texture = self.widget_textures.entry(id).or_insert_with(|| {
                WidgetTexture::new(
                    id,
                    width,
                    height,
                    device,
                    queue,
                    &self.texture_bind_group_layout,
                )
            });

            widget_manager.draw(id, widget_texture.frame_mut(), width, height);

            queue.write_texture(
                // Tells wgpu where to copy the pixel data
                wgpu::ImageCopyTexture {
                    texture: &widget_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                // The actual pixel data
                &widget_texture.frame(),
                // The layout of the texture
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(widget_texture.size.width * 4),
                    rows_per_image: Some(widget_texture.size.height),
                },
                widget_texture.size,
            );

            let mut widgets_builder = WidgetQuadBufferBuilder::new();

            for &(_, quad) in group {
                widgets_builder.push_quad(quad);
            }

            let (stg_vertex, stg_index, widgets_num_indices) = widgets_builder.build(&device);

            stg_vertex.copy_to_buffer(encoder, &widget_texture.vertex_buffer);
            stg_index.copy_to_buffer(encoder, &widget_texture.index_buffer);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Widget #{id} render pass")),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &system.bind_group, &[]);
            render_pass.set_bind_group(1, &widget_texture.bind_group, &[]);
            render_pass.set_vertex_buffer(0, widget_texture.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                widget_texture.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..widgets_num_indices, 0, 0..1);
        }
    }
}

pub struct WidgetTexture {
    texture: wgpu::Texture,
    size: wgpu::Extent3d,
    // texture_view: wgpu::TextureView,
    // sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    pixels: Vec<u8>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl WidgetTexture {
    pub fn new(
        id: usize,
        width: usize,
        height: usize,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // let diffuse_bytes = include_bytes!("../../res/example_waveform.png");
        // let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        // let diffuse_rgba = diffuse_image.to_rgba8();
        // let dimensions = diffuse_image.dimensions();

        let size = wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Widget #{id} pixel texture")),
            size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // buffer size
        // See [https://github.com/parasyte/pixels/blob/main/src/builder.rs]
        // 32-bit formats, 8 bits per component
        let texture_format_size = 4;
        let pixels_buffer_size = (width * height * texture_format_size) as usize;

        let mut pixels = Vec::with_capacity(pixels_buffer_size);
        pixels.resize_with(pixels_buffer_size, Default::default);

        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Texture bind group"),
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Widgets vertex buffer"),
            size: WidgetVertex::SIZE * 400,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Widgets index buffer"),
            size: WidgetVertex::SIZE * 400,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            texture,
            size,
            // texture_view,
            // sampler,
            bind_group,
            pixels,

            vertex_buffer,
            index_buffer,
        }
    }

    // pub fn draw(&mut self) {
    //     queue.write_texture(
    //         // Tells wgpu where to copy the pixel data
    //         wgpu::ImageCopyTexture {
    //             texture: &texture,
    //             mip_level: 0,
    //             origin: wgpu::Origin3d::ZERO,
    //             aspect: wgpu::TextureAspect::All,
    //         },
    //         // The actual pixel data
    //         &diffuse_rgba,
    //         // The layout of the texture
    //         wgpu::ImageDataLayout {
    //             offset: 0,
    //             bytes_per_row: Some(4 * dimensions.0),
    //             rows_per_image: Some(dimensions.1),
    //         },
    //         texture_size,
    //     );
    // }

    /// Get a mutable byte slice for the pixel buffer. The buffer is _not_ cleared for you; it will
    /// retain the previous frame's contents until you clear it yourself.
    pub fn frame_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Get an immutable byte slice for the pixel buffer.
    ///
    /// This may be useful for operations that must sample the buffer, such as blending pixel
    /// colours directly into it.
    pub fn frame(&self) -> &[u8] {
        &self.pixels
    }
}
