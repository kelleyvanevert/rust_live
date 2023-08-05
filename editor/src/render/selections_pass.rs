use live_editor_state::{EditorState, LineSelection, Pos};

use super::{
    buffer::{QuadBufferBuilder, Vertex},
    system::SystemData,
};

pub struct SelectionsPass {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl SelectionsPass {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        system: &SystemData,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shader.wgsl").into()),
        });

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

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn resize(&mut self, _queue: &wgpu::Queue, _config: &wgpu::SurfaceConfiguration) {}

    pub fn draw<'pass>(
        &'pass mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        system: &'pass SystemData,
        editor_state: &EditorState,
        render_pass: &mut wgpu::RenderPass<'pass>,
    ) {
        let sf = system.scale_factor;

        let mut builder = QuadBufferBuilder::new();

        for LineSelection {
            row,
            col_start,
            col_end,
        } in editor_state.visual_selections()
        {
            let (x_start, y) = system.pos_to_px(Pos {
                row,
                col: col_start,
            });

            let (x_end, _) = system.pos_to_px(Pos { row, col: col_end });

            builder.push_quad(
                x_start,
                y,
                x_end + 6.0 / sf,
                y + system.char_size.1 / sf,
                [0.0, 0.0, 0.0, 0.2],
            );
        }

        for caret in editor_state.caret_positions() {
            let (cx, cy) = system.pos_to_px(caret);

            builder.push_quad(
                cx,
                cy,
                cx + 6.0 / sf,
                cy + system.char_size.1 / sf,
                [0.0, 0.0, 0.0, 1.0],
            );
        }

        let vertex_data_raw: &[u8] = bytemuck::cast_slice(&builder.vertex_data);
        queue.write_buffer(&self.vertex_buffer, 0, vertex_data_raw);

        let index_data_raw: &[u8] = bytemuck::cast_slice(&builder.index_data);
        queue.write_buffer(&self.index_buffer, 0, index_data_raw);

        let num_indices = builder.num_indices();

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &system.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32); // 1.
        render_pass.draw_indexed(0..num_indices, 0, 0..1); // 2.
    }
}
