mod background_pass;
mod buffer;
mod code_pass;
mod pass;
mod selections_pass;
mod system;
mod widget_vertex;
mod widgets_pass;

use crate::widget::WidgetManager;

use self::{
    background_pass::BackgroundPass, code_pass::CodePass, selections_pass::SelectionsPass,
    system::SystemData, widgets_pass::WidgetsPass,
};
use live_editor_state::EditorState;
use winit::dpi::PhysicalSize;

pub struct Renderer<'a> {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,

    pub system: SystemData,

    background_pass: BackgroundPass,
    code_pass: CodePass<'a>,
    widgets_pass: WidgetsPass,
    selections_pass: SelectionsPass,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &winit::window::Window) -> Renderer<'a> {
        let scale_factor = window.scale_factor() as f32;

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

        let background_pass = BackgroundPass::new();
        let code_pass = CodePass::new(&device, &queue, &config);
        let system = SystemData::new(
            scale_factor,
            code_pass.char_size(),
            &device,
            &queue,
            &config,
        );
        let widgets_pass = WidgetsPass::new(&device, &queue, &config, &system);
        let selections_pass = SelectionsPass::new(&device, &queue, &config, &system);

        Self {
            device,
            queue,
            surface,
            config,

            system,
            background_pass,
            widgets_pass,
            code_pass,
            selections_pass,
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

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);

        self.surface.configure(&self.device, &self.config);
        self.system.resize(&self.queue, &self.config);
        self.code_pass.resize(&self.queue, &self.config);
        self.selections_pass.resize(&self.queue, &self.config);
    }

    pub fn draw(&mut self, editor_state: &EditorState, widget_manager: &mut WidgetManager) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture!");

        let view = frame.texture.create_view(&Default::default());

        self.background_pass.draw(&view, &mut encoder);

        let widget_instances = self.code_pass.draw(
            &self.device,
            &self.queue,
            &self.system,
            &view,
            editor_state,
            &mut encoder,
        );

        self.widgets_pass.draw(
            &self.device,
            &self.queue,
            &self.system,
            &view,
            widget_instances,
            widget_manager,
            &mut encoder,
        );

        self.selections_pass.draw(
            &self.device,
            &self.queue,
            &self.system,
            &view,
            editor_state,
            &mut encoder,
        );

        self.queue.submit([encoder.finish()]);

        frame.present();
    }
}
