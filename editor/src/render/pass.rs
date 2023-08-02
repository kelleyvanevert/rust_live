pub trait Pass {
    fn draw(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // obj_model: &Model,
    ) -> Result<(), wgpu::SurfaceError>;
}
