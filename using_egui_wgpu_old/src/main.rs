#![feature(let_chains)]
#![feature(if_let_guard)]

use std::iter;
use std::time::{Duration, Instant, SystemTime};

use app::App;
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::dpi::{LogicalSize, Size};
use winit::event::Event::*;
use winit::event_loop::ControlFlow;
use winit::platform::macos::WindowBuilderExtMacOS;

mod app;
mod egui_wgpu_backend;
mod egui_winit_platform;
mod read_audio_file;
mod syntax_highlighting;

enum CustomEvent {}

/// A simple egui + wgpu + winit based example.
fn main() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoopBuilder::<CustomEvent>::with_user_event().build();
    let window = winit::window::WindowBuilder::new()
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

    let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ))
    .unwrap();

    let size = window.inner_size();

    let surface_caps = surface.get_capabilities(&adapter);

    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .filter(|f| f.is_srgb())
        .next()
        .unwrap_or(surface_caps.formats[0]);

    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    // let mut surface_config = surface
    //     .get_default_config(&adapter, size.width, size.height)
    //     .expect("Surface isn't supported by the adapter.");

    surface.configure(&device, &surface_config);

    // We use the egui_winit_platform crate as the platform.
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
        window_drag_surface_height: Some(App::WINDOW_DRAG_SURFACE_HEIGHT),
    });

    // We use the egui_wgpu_backend crate as the render backend.
    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);

    // Display the demo application that ships with egui.
    let mut app = App::new(&platform.context());

    // // FPS and window updating:
    // let mut frameno = 0;
    // let mut then = SystemTime::now();
    // let mut now = SystemTime::now();
    // let mut fps = 0;
    // // change '60.0' if you want different FPS cap
    // let target_framerate = Duration::from_secs_f64(1.0 / 60.0);
    // let mut delta_time = Instant::now();

    let start_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        platform.handle_event(&window, &event);

        match event {
            RedrawRequested(..) => {
                // frameno += 1;
                // fps += 1;
                // if now.duration_since(then).unwrap().as_millis() > 1000 {
                //     window.set_title(&format!("Frame {}, FPS: {}", frameno, fps));
                //     fps = 0;
                //     then = now;
                // }
                // now = SystemTime::now();

                platform.update_time(start_time.elapsed().as_secs_f64());

                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        // This error occurs when the app is minimized on Windows.
                        // Silently return here to prevent spamming the console with:
                        // "The underlying surface has changed, and therefore the swap chain must be updated"
                        return;
                    }
                    Err(e) => {
                        eprintln!("Dropped frame with error: {}", e);
                        return;
                    }
                };
                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Begin to draw the UI frame.
                platform.begin_frame();

                app.begin_frame();

                // Draw the demo application.
                let _response = app.ui(&platform.context());

                // End the UI frame. We could now handle the output and draw the UI with the backend.
                let full_output = platform.end_frame(Some(&window));
                let paint_jobs = platform.context().tessellate(full_output.shapes);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

                // Upload all resources for the GPU.
                let screen_descriptor = ScreenDescriptor {
                    physical_width: surface_config.width,
                    physical_height: surface_config.height,
                    scale_factor: window.scale_factor() as f32,
                };
                let tdelta: egui::TexturesDelta = full_output.textures_delta;
                egui_rpass
                    .add_textures(&device, &queue, &tdelta)
                    .expect("add texture ok");
                egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                // Record all render passes.
                egui_rpass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        Some(wgpu::Color::BLACK),
                    )
                    .unwrap();

                // Submit the commands.
                queue.submit(iter::once(encoder.finish()));

                // Redraw egui
                output_frame.present();

                egui_rpass
                    .remove_textures(tdelta)
                    .expect("remove texture ok");

                // Support reactive on windows only, but not on linux.
                // if _output.needs_repaint {
                //     *control_flow = ControlFlow::Poll;
                // } else {
                //     *control_flow = ControlFlow::Wait;
                // }
            }
            MainEventsCleared => {
                window.request_redraw();
                // if target_framerate <= delta_time.elapsed() {
                //     window.request_redraw();
                //     delta_time = Instant::now();
                // } else {
                //     *control_flow = ControlFlow::WaitUntil(
                //         Instant::now().checked_sub(delta_time.elapsed()).unwrap()
                //             + target_framerate,
                //     );
                // }
            }
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                    // See: https://github.com/rust-windowing/winit/issues/208
                    // This solves an issue where the app would panic when minimizing on Windows.
                    if size.width > 0 && size.height > 0 {
                        surface_config.width = size.width;
                        surface_config.height = size.height;
                        surface.configure(&device, &surface_config);
                    }
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => (),
        }
    });
}
