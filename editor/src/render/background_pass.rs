use wgpu::TextureView;

const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 243.0 / 255.0,
    g: 242.0 / 255.0,
    b: 240.0 / 255.0,
    a: 1.,
};

pub struct BackgroundPass {}

impl BackgroundPass {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&mut self, view: &TextureView, encoder: &mut wgpu::CommandEncoder) {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Background render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,

                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
    }
}
