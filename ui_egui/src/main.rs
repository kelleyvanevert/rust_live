use std::time::{Duration, Instant, SystemTime};

use egui_winit::State;
use winit::{
    dpi::{LogicalSize, Size},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    keyboard::Key,
    platform::macos::WindowBuilderExtMacOS,
    window::WindowBuilder,
};

mod egui_winit;

fn main() {
    env_logger::init();

    let event_loop = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title("")
        .with_fullsize_content_view(true)
        .with_titlebar_transparent(true)
        .with_active(true)
        .with_inner_size(Size::Logical(LogicalSize {
            width: 1000.0,
            height: 700.0,
        }))
        .with_resizable(true)
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window);

    // let mut renderer = pollster::block_on(Renderer::new(&window));

    // FPS and window updating:
    let mut frameno: u64 = 0;
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
                    // let s = window.inner_size().to_logical::<f64>(window.scale_factor());
                    // width = s.width;
                    // height = s.height;

                    // state.resize(&window);
                    // renderer.update(&state);
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
                WindowEvent::CursorMoved { position, .. } => {
                    // let p = position.to_logical::<f64>(window.scale_factor());

                    // let _ = frontend.send(Params {
                    //     frequency: Some(220.0 + 440.0 * (p.x / width) as f32),
                    //     squareness: Some((p.y / height) as f32),
                    //     volume: None,
                    // });
                }
                _ => (),
            },
            winit::event::Event::RedrawRequested(_) => {
                frameno += 1;

                state.take_egui_input(&window);
                // state.handle_platform_output(&window, egui_ctx, platform_output)

                // renderer.update(&state);
                // renderer.draw(&state);

                fps += 1;
                if now.duration_since(then).unwrap().as_millis() > 1000 {
                    window.set_title(&format!("Frame {}, FPS: {}", frameno, fps));
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
