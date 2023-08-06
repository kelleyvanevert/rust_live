#![feature(let_chains)]
#![feature(slice_group_by)]

use std::time::{Duration, Instant, SystemTime};
use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::event::KeyEvent;
use winit::event_loop::EventLoopBuilder;
use winit::platform::macos::WindowBuilderExtMacOS;
use winit::window::Window;
use winit::{
    event::{ElementState, WindowEvent},
    event_loop::ControlFlow,
    keyboard::Key,
    window::WindowBuilder,
};

pub fn main() {
    env_logger::init();

    let event_loop = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title("")
        .with_fullsize_content_view(true)
        .with_titlebar_transparent(true)
        .with_active(true)
        .with_inner_size(Size::Logical(LogicalSize {
            width: 900.0,
            height: 600.0,
        }))
        .with_resizable(true)
        .build(&event_loop)
        .unwrap();

    let mut state = State::new();

    let mut renderer = pollster::block_on(Renderer::new(&window));

    // FPS and window updating:
    let mut then = SystemTime::now();
    let mut now = SystemTime::now();
    let mut fps = 0;
    // change '60.0' if you want different FPS cap
    let target_framerate = Duration::from_secs_f64(1.0 / 60.0);
    let mut delta_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut size,
                    ..
                } => {
                    renderer.resize(size);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state, logical_key, ..
                        },
                    ..
                } => match (logical_key.clone(), state) {
                    (Key::Escape, ElementState::Pressed) => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                },
                _ => (),
            },
            winit::event::Event::RedrawRequested(_) => {
                state.frameno += 1;

                renderer.draw(&state);

                fps += 1;
                if now.duration_since(then).unwrap().as_millis() > 1000 {
                    window.set_title(&format!("Frame {}, FPS: {}", state.frameno, fps));
                    fps = 0;
                    then = now;
                }
                now = SystemTime::now();
            }
            winit::event::Event::MainEventsCleared => {
                if target_framerate <= delta_time.elapsed() {
                    window.request_redraw();
                    delta_time = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now().checked_sub(delta_time.elapsed()).unwrap()
                            + target_framerate,
                    );
                }
            }
            _ => (),
        }
    });
}

struct State {
    t0: Instant,
    frameno: usize,
}

impl State {
    fn new() -> Self {
        Self {
            t0: Instant::now(),
            frameno: 0,
        }
    }
}

struct Renderer {
    scale_factor: f32,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    async fn new(window: &Window) -> Self {
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

        Self {
            scale_factor,
            surface,
            config,
            device,
            queue,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);

        self.surface.configure(&self.device, &self.config);
        // self.system.resize(&self.queue, &self.config);
        // self.code_pass.resize(&self.queue, &self.config);
        // self.selections_pass.resize(&self.queue, &self.config);
    }

    #[allow(unused)]
    pub fn width(&self) -> f32 {
        self.config.width as f32
    }

    #[allow(unused)]
    pub fn height(&self) -> f32 {
        self.config.height as f32
    }

    fn draw(&mut self, state: &State) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture!");

        let view = frame.texture.create_view(&Default::default());

        // self.background_pass.draw(&view, &mut encoder);

        // self.widget_instances = self.code_pass.draw(
        //     &self.device,
        //     &self.queue,
        //     &self.system,
        //     &view,
        //     editor_state,
        //     &mut encoder,
        // );

        // self.widgets_pass.draw(
        //     &self.device,
        //     &self.queue,
        //     &self.system,
        //     &view,
        //     &self.widget_instances,
        //     widget_manager,
        //     &mut encoder,
        // );

        // self.selections_pass.draw(
        //     &self.device,
        //     &self.queue,
        //     &self.system,
        //     &view,
        //     editor_state,
        //     &mut encoder,
        // );

        self.queue.submit([encoder.finish()]);

        frame.present();
    }
}
