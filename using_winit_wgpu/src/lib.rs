#![feature(let_chains)]

mod highlight;
mod render;
mod util;

use live_editor_state::{Direction, EditorState, LineData, Pos, Token};
use std::time::{Duration, Instant, SystemTime};
use winit::dpi::{LogicalPosition, LogicalSize, Size};
use winit::event::{KeyEvent, MouseButton};
use winit::platform::macos::WindowBuilderExtMacOS;
use winit::{
    event::{ElementState, WindowEvent},
    event_loop::{self, ControlFlow},
    keyboard::Key,
    window::WindowBuilder,
};

pub fn run() {
    env_logger::init();

    let event_loop = event_loop::EventLoop::new();
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

    let mut editor_state = EditorState::new().with_linedata(
        LineData::from(
            "A kelley wrote
  some
  code that' eventually
  and bla bla bla bla bla

run off
  the screen
  and bla bla bla
",
        )
        .with_widget_at_pos(Pos { row: 2, col: 12 }, 0, 5)
        .with_widget_at_pos(Pos { row: 6, col: 7 }, 1, 4)
        .with_inserted(Pos { row: 0, col: 5 }, LineData::from("hi\nthere kelley ")),
    );

    let mut is_selecting: Option<usize> = None;
    let mut shift_pressed = false;
    let mut alt_pressed = false;
    let mut meta_or_ctrl_pressed = false;
    let mut mouse_at: Option<(f32, f32)> = None;

    let mut render = pollster::block_on(render::Render::new(&window));

    let mut apply_shader_pipeline = true;

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
                    render.resize(size);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state,
                            logical_key,
                            ..
                        },
                    ..
                } => match (logical_key.clone(), state) {
                    (Key::Delete, ElementState::Pressed) => {
                        editor_state.clear()
                    },
                    // (Key::GoBack, ElementState::Pressed) if !code_section.text.is_empty() => {
                    //     let mut end_text = code_section.text.remove(code_section.text.len() - 1);
                    //     end_text.text.pop();
                    //     if !end_text.text.is_empty() {
                    //         code_section.text.push(end_text);
                    //     }
                    // }
                    (Key::Tab, ElementState::Pressed) => {
                        if shift_pressed {
                            editor_state.untab();
                        } else {
                            editor_state.tab();
                        }
                    },
                    (Key::Space, ElementState::Pressed) => {
                        editor_state.write(" ");
                    }
                    (Key::Enter, ElementState::Pressed) => {
                        if meta_or_ctrl_pressed {
                            apply_shader_pipeline = !apply_shader_pipeline;
                        } else {
                            editor_state.write("\n");
                        }
                    }
                    (Key::Backspace, ElementState::Pressed) => {
                        editor_state.backspace();
                    }
                    (Key::ArrowUp, ElementState::Pressed) => {
                        editor_state.move_caret(Direction::Up, shift_pressed);
                    },
                    (Key::ArrowRight, ElementState::Pressed) => {
                        editor_state.move_caret(Direction::Right, shift_pressed);
                    },
                    (Key::ArrowDown, ElementState::Pressed) => {
                        editor_state.move_caret(Direction::Down, shift_pressed);
                    },
                    (Key::ArrowLeft, ElementState::Pressed) => {
                        editor_state.move_caret(Direction::Left, shift_pressed);
                    },
                    (Key::Character(s), ElementState::Pressed) => {
                        if s.as_str() == "a" && meta_or_ctrl_pressed {
                            editor_state.select_all();
                        } else {
                            editor_state.write(s.as_str());
                        }
                    }
                    (Key::Alt, ElementState::Pressed) => {
                        alt_pressed = true;
                    }
                    (Key::Alt, ElementState::Released) => {
                        alt_pressed = false;
                    }
                    (Key::Shift, ElementState::Pressed) => {
                        shift_pressed = true;
                    }
                    (Key::Shift, ElementState::Released) => {
                        shift_pressed = false;
                    }
                    (Key::Meta, ElementState::Pressed) => {
                        meta_or_ctrl_pressed = true;
                    }
                    (Key::Meta, ElementState::Released) => {
                        meta_or_ctrl_pressed = false;
                    }
                    (Key::Super, ElementState::Pressed) => {
                        meta_or_ctrl_pressed = true;
                    }
                    (Key::Super, ElementState::Released) => {
                        meta_or_ctrl_pressed = false;
                    }
                    (Key::Control, ElementState::Pressed) => {
                        meta_or_ctrl_pressed = true;
                    }
                    (Key::Control, ElementState::Released) => {
                        meta_or_ctrl_pressed = false;
                    }
                    _ => {
                        // println!("key: {:?}, state: {:?}", logical_key, state);
                    }
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    if let Some(p) = mouse_at && state == ElementState::Pressed && button == MouseButton::Left {
                        let pos = render.px_to_pos(p);
                        if alt_pressed {
                            is_selecting = Some(editor_state.add_caret(pos));
                        } else{
                            is_selecting = Some(editor_state.set_single_caret(pos));
                        }
                    } else if state == ElementState::Released && button == MouseButton::Left {
                        is_selecting = None;
                    }
                }
                WindowEvent::CursorEntered { .. } => {
                    println!("cursor entered");
                }
                WindowEvent::CursorLeft { .. } => {
                    println!("cursor left");
                    mouse_at = None;
                    // is_selecting = false;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let position: LogicalPosition<f32> = position.to_logical(render.scale_factor.into());
                    let p = (position.x as f32, position.y as f32);
                    mouse_at = Some(p);

                    if let Some(id) = is_selecting {
                        let caret = render.px_to_pos(p);
                        editor_state.drag_select(caret, id);
                    }
                }
                WindowEvent::Moved(u) => {
                    println!("moved {:?}", u);
                }
                WindowEvent::DragEnter { paths, position } => {
                    // println!("drag enter {:?}", position);
                    // for path in paths {
                    //     println!("  - {:?}", path);
                    // }
                }
                WindowEvent::DragOver { position } => {
                    let position: LogicalPosition<f32> = position.to_logical(render.scale_factor.into());
                    let pos = render.px_to_pos((position.x as f32, position.y as f32));

                    editor_state.file_drag_hover(pos);
                }
                WindowEvent::DragDrop { paths, position } => {
                    let position: LogicalPosition<f32> = position.to_logical(render.scale_factor.into());
                    let pos = render.px_to_pos((position.x as f32, position.y as f32));

                    editor_state.insert(pos, Token::Widget { id: 0, width: 5 }.into(), true);
                }
                _ => (),
            },
            winit::event::Event::RedrawRequested(_) => {
                render.render_state(&editor_state, apply_shader_pipeline);
                // if state.game_state != state::GameState::Quiting {
                window.request_redraw();
                // }

                fps += 1;
                if now.duration_since(then).unwrap().as_millis() > 1000 {
                    window.set_title(&format!("FPS: {}", fps));
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

// fn lerp(v: f32, domain: (f32, f32), range: (f32, f32)) -> f32 {
//     (v - domain.0) * (range.1 - range.0) / (domain.1 - domain.0)
// }
