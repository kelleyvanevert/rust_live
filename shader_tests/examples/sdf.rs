#![feature(let_chains)]
#![feature(slice_group_by)]

use cgmath::SquareMatrix;
use std::time::{Duration, Instant, SystemTime};
use wgpu::util::DeviceExt;
use winit::dpi::{LogicalSize, Size};
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

    let mut state = State::new(&window);

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
                WindowEvent::Resized(_)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut _,
                    ..
                } => {
                    state.resize(&window);
                    renderer.update(&state);
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

                renderer.update(&state);
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

pub struct State {
    t0: Instant,
    frameno: usize,

    width: f32,
    height: f32,
}

impl State {
    fn new(window: &Window) -> Self {
        Self {
            t0: Instant::now(),
            frameno: 0,
            width: window
                .inner_size()
                .to_logical::<f32>(window.scale_factor())
                .width as f32,
            height: window
                .inner_size()
                .to_logical::<f32>(window.scale_factor())
                .height as f32,
        }
    }

    fn resize(&mut self, window: &Window) {
        self.width = window
            .inner_size()
            .to_logical::<f32>(window.scale_factor())
            .width as f32;

        self.height = window
            .inner_size()
            .to_logical::<f32>(window.scale_factor())
            .height as f32;
    }
}

/**
   System global stuff, like the projection matrix and coordinate stuff
*/
pub struct SystemData {
    pub scale_factor: f32,
    pub system_uniform: SystemUniform,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl SystemData {
    pub fn new(
        scale_factor: f32,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let mut system_uniform = SystemUniform::new(config.width as f32, config.height as f32);
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
            system_uniform,
            bind_group_layout,
            bind_group,
            buffer,
        }
    }

    fn update_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.system_uniform]),
        );
    }

    pub fn update_for_state(&mut self, queue: &wgpu::Queue, state: &State) {
        self.system_uniform.dim = [state.width, state.height];

        self.system_uniform
            .update(self.scale_factor, (state.width, state.height));

        self.system_uniform.time = state.t0.elapsed().as_secs_f32();

        self.update_buffer(&queue);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SystemUniform {
    view_proj: [[f32; 4]; 4],

    time: f32,
    // frameno: f32,
    dim: [f32; 2],
}

impl SystemUniform {
    fn new(width: f32, height: f32) -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            time: 0.0,
            dim: [width, height],
        }
    }

    fn update(&mut self, _sf: f32, (width, height): (f32, f32)) {
        // because now, the width and height are logical instead of physical..
        // TODO, when building something real: only use `PhysicalSize` and `LogicalSize`, to be sure of what we're working with..
        let sf = 1.0;

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

struct Renderer {
    #[allow(unused)]
    scale_factor: f32,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,

    system: SystemData,
    sdf_pass: SdfPass,
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

        let system = SystemData::new(scale_factor, &device, &queue, &config);

        let sdf_pass = SdfPass::new(&device, &queue, &config, &system);

        Self {
            scale_factor,
            surface,
            config,
            device,
            queue,

            system,
            sdf_pass,
        }
    }

    // fn resize(&mut self, size: PhysicalSize<u32>) {
    //     self.config.width = size.width.max(1);
    //     self.config.height = size.height.max(1);

    //     self.surface.configure(&self.device, &self.config);
    //     // self.system.resize(&self.queue, &self.config);
    //     self.sdf_pass.resize(&self.queue, &self.config);
    // }

    fn update(&mut self, state: &State) {
        self.config.width = state.width.round() as u32 * 2;
        self.config.height = state.height.round() as u32 * 2;

        self.surface.configure(&self.device, &self.config);
        // self.system.resize(&self.queue, &self.config);
        self.sdf_pass.resize(&self.queue, &self.config);

        self.system.update_for_state(&self.queue, state);
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

        {
            let background_color = wgpu::Color {
                r: 243.0 / 255.0,
                g: 242.0 / 255.0,
                b: 240.0 / 255.0,
                a: 1.,
            };

            let view = frame.texture.create_view(&Default::default());

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(background_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.sdf_pass.draw(
                &self.device,
                &self.queue,
                &self.system,
                state,
                &mut render_pass,
            );
        }
        self.queue.submit([encoder.finish()]);

        frame.present();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    // color: [f32; 4],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;

    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
        0 => Float32x3,
        // 1 => Float32x4,
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    fn from((x, y): (f32, f32)) -> Self {
        Self {
            position: [x, y, 0.0],
        }
    }
}

struct VertexBufferBuilder {
    vertex_data: Vec<Vertex>,
    index_data: Vec<u32>,
}

impl VertexBufferBuilder {
    fn new() -> Self {
        Self {
            vertex_data: Vec::new(),
            index_data: Vec::new(),
        }
    }

    fn push_triangle(&mut self, a: (f32, f32), b: (f32, f32), c: (f32, f32)) {
        let num_vertices = self.vertex_data.len() as u32;

        self.vertex_data.extend(&[
            //
            Vertex::from(a),
            Vertex::from(b),
            Vertex::from(c),
        ]);

        self.index_data.extend(&[
            //
            num_vertices + 0,
            num_vertices + 1,
            num_vertices + 2,
        ]);
    }

    // pub fn push_quad(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32, color: [f32; 4]) {
    //     self.vertex_data.extend(&[
    //         Vertex {
    //             position: [min_x, min_y, 0.0],
    //             color,
    //         },
    //         Vertex {
    //             position: [max_x, min_y, 0.0],
    //             color,
    //         },
    //         Vertex {
    //             position: [max_x, max_y, 0.0],
    //             color,
    //         },
    //         Vertex {
    //             position: [min_x, max_y, 0.0],
    //             color,
    //         },
    //     ]);
    //     self.index_data.extend(&[
    //         self.current_quad * 4 + 0,
    //         self.current_quad * 4 + 1,
    //         self.current_quad * 4 + 2,
    //         //
    //         self.current_quad * 4 + 0,
    //         self.current_quad * 4 + 2,
    //         self.current_quad * 4 + 3,
    //     ]);
    //     self.current_quad += 1;
    // }

    pub fn num_indices(&self) -> u32 {
        self.index_data.len() as u32
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
        system: &SystemData,
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
        state: &State,
        render_pass: &mut wgpu::RenderPass<'pass>,
    ) {
        let mut builder = VertexBufferBuilder::new();

        builder.push_triangle(
            (50.0, 50.0),
            (state.width - 50.0, 50.0),
            (50.0, state.height - 50.0),
        );

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
