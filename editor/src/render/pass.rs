pub trait Pass {
    fn draw(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        system_bind_group: &wgpu::BindGroup,
        // obj_model: &Model,
    ) -> Result<(), wgpu::SurfaceError>;
}
