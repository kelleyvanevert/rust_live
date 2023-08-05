#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;

    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
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

pub struct QuadBufferBuilder {
    pub vertex_data: Vec<Vertex>,
    pub index_data: Vec<u32>,
    pub current_quad: u32,
}

impl QuadBufferBuilder {
    pub fn new() -> Self {
        Self {
            vertex_data: Vec::new(),
            index_data: Vec::new(),
            current_quad: 0,
        }
    }

    pub fn push_quad(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32, color: [f32; 4]) {
        self.vertex_data.extend(&[
            Vertex {
                position: [min_x, min_y, 0.0],
                color,
            },
            Vertex {
                position: [max_x, min_y, 0.0],
                color,
            },
            Vertex {
                position: [max_x, max_y, 0.0],
                color,
            },
            Vertex {
                position: [min_x, max_y, 0.0],
                color,
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
