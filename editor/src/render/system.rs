use cgmath::SquareMatrix;
use live_editor_state::Pos;
use wgpu::util::DeviceExt;

/**
   System global stuff, like the projection matrix and coordinate stuff
*/
pub struct SystemData {
    pub scale_factor: f32,
    pub char_size: (f32, f32),

    pub system_uniform: SystemUniform,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl SystemData {
    pub fn new(
        scale_factor: f32,
        char_size: (f32, f32),
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let mut system_uniform = SystemUniform::new();
        system_uniform.update(scale_factor, (config.width as f32, config.height as f32));

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("System buffer"),
            contents: bytemuck::cast_slice(&[system_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                // wgpu::BindGroupEntry {
                //     binding: 1,
                //     resource: wgpu::BindingResource::TextureView(&texture_view),
                // },
            ],
            label: None,
        });

        Self {
            scale_factor,
            char_size,

            system_uniform,
            bind_group_layout,
            bind_group,
            buffer,
        }
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

    pub fn px_to_pos_f(&self, (x, y): (f32, f32)) -> Pos<f32> {
        let sf = self.scale_factor;
        Pos {
            row: ((y * sf - 260.0) / self.char_size.1),
            col: ((x * sf - 100.0) / self.char_size.0),
        }
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.system_uniform.update(
            self.scale_factor,
            (config.width as f32, config.height as f32),
        );

        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.system_uniform]),
        );
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SystemUniform {
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
