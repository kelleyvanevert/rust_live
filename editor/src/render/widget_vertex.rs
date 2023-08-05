#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WidgetVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

unsafe impl bytemuck::Pod for WidgetVertex {}
unsafe impl bytemuck::Zeroable for WidgetVertex {}

impl WidgetVertex {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;

    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct WidgetQuadBufferBuilder {
    pub vertex_data: Vec<WidgetVertex>,
    pub index_data: Vec<u32>,
    pub current_quad: u32,
}

impl WidgetQuadBufferBuilder {
    pub fn new() -> Self {
        Self {
            vertex_data: Vec::new(),
            index_data: Vec::new(),
            current_quad: 0,
        }
    }

    pub fn push_quad(&mut self, (min_x, min_y, max_x, max_y): (f32, f32, f32, f32)) {
        self.vertex_data.extend(&[
            WidgetVertex {
                position: [min_x, min_y, 0.0],
                tex_coords: [0.0, 0.0],
            },
            WidgetVertex {
                position: [max_x, min_y, 0.0],
                tex_coords: [1.0, 0.0],
            },
            WidgetVertex {
                position: [max_x, max_y, 0.0],
                tex_coords: [1.0, 1.0],
            },
            WidgetVertex {
                position: [min_x, max_y, 0.0],
                tex_coords: [0.0, 1.0],
            },
        ]);
        self.index_data.extend(&[
            self.current_quad * 4 + 0,
            self.current_quad * 4 + 1,
            self.current_quad * 4 + 2,
            //
            self.current_quad * 4 + 0,
            self.current_quad * 4 + 2,
            self.current_quad * 4 + 3,
        ]);
        self.current_quad += 1;
    }

    pub fn num_indices(&self) -> u32 {
        self.index_data.len() as u32
    }
}
