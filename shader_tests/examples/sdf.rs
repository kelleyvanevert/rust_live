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

pub struct SdfPass {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl SdfPass {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        // system: &SystemData,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./sdf.wgsl").into()),
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
            fragment: None,
            // fragment: Some(wgpu::FragmentState {
            //     // 3.
            //     module: &shader,
            //     entry_point: "fs_main",
            //     targets: &[Some(wgpu::ColorTargetState {
            //         // 4.
            //         format: config.format,
            //         write_mask: wgpu::ColorWrites::ALL,
            //         blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            //     })],
            // }),
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

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        // system: &SystemData,
        view: &wgpu::TextureView,
        // editor_state: &EditorState,
        encoder: &mut wgpu::CommandEncoder,
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

        let (stg_vertex, stg_index, num_indices) = builder.build(&device);

        stg_vertex.copy_to_buffer(encoder, &self.vertex_buffer);
        stg_index.copy_to_buffer(encoder, &self.index_buffer);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Selections render pass"),
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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32); // 1.
        render_pass.draw_indexed(0..num_indices, 0, 0..1); // 2.
    }
}
