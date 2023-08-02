#![feature(let_chains)]

mod clipboard;
mod highlight;
mod render;
mod util;
mod widget;
mod widgets;

use clipboard::Clipboard;
use live_editor_state::{Direction, EditorState, LineData, MoveVariant, Pos, Token};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};
use widget::Widget;
use widgets::sample::SampleWidget;
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

    let sample_widget_0 = SampleWidget::new(PathBuf::from("12345"));
    let sample_widget_1 = SampleWidget::new(PathBuf::from("1234"));

    let linedata = LineData::from(
        "A kelley wrote
  some
  code that' eventually
  and bla bla bla bla bla

run off
  the screen
  and bla bla bla
",
    )
    .with_widget_at_pos(Pos { row: 2, col: 12 }, 0, sample_widget_0.column_width())
    .with_widget_at_pos(Pos { row: 6, col: 7 }, 1, sample_widget_1.column_width())
    .with_inserted(Pos { row: 0, col: 5 }, LineData::from("hi\nthere kelley "));

    let mut widgets: Vec<Box<dyn Widget>> =
        vec![Box::new(sample_widget_0), Box::new(sample_widget_1)];

    let mut editor_state = EditorState::new().with_linedata(linedata);

    let mut is_selecting: Option<usize> = None;
    let mut shift_pressed = false;
    let mut alt_pressed = false;
    let mut meta_or_ctrl_pressed = false;
    let mut mouse_at: Option<(f32, f32)> = None;

    let mut render = pollster::block_on(render::Renderer::new(&window));

    let mut clipboard = Clipboard::new();

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
                    // (Key::Escape, ElementState::Pressed) => {
                    //     *control_flow = ControlFlow::Exit;
                    // },
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
                        editor_state.write("\n");
                    }
                    (Key::Backspace, ElementState::Pressed) => {
                        editor_state.backspace(if alt_pressed {
                            MoveVariant::ByWord
                        } else if meta_or_ctrl_pressed {
                            MoveVariant::UntilEnd
                        } else {
                            MoveVariant::ByToken
                        });
                    }
                    (Key::ArrowUp | Key::ArrowRight | Key::ArrowDown | Key::ArrowLeft, ElementState::Pressed) => {
                        editor_state.move_caret(
                            match logical_key.clone() {
                                Key::ArrowUp => Direction::Up,
                                Key::ArrowRight => Direction::Right,
                                Key::ArrowDown => Direction::Down,
                                Key::ArrowLeft => Direction::Left,
                                _ => unreachable!()
                            },
                            shift_pressed,
                            if alt_pressed {
                                MoveVariant::ByWord
                            } else if meta_or_ctrl_pressed {
                                MoveVariant::UntilEnd
                            } else {
                                MoveVariant::ByToken
                            },
                        );
                    },
                    (Key::Character(s), ElementState::Pressed) => {
                        if s.as_str() == "c" && meta_or_ctrl_pressed {
                            // todo improve (ctrl/meta depending on OS)
                            clipboard.write(editor_state.copy());
                        } else if s.as_str() == "x" && meta_or_ctrl_pressed {
                            // todo improve (ctrl/meta depending on OS)
                            clipboard.write(editor_state.cut());
                        } else if s.as_str() == "v" && meta_or_ctrl_pressed {
                            // todo improve (ctrl/meta depending on OS)
                            if let Some(data) = clipboard.read() {
                                editor_state.paste(data);
                            }
                        } else if s.as_str() == "a" && meta_or_ctrl_pressed {
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
                        let pos = render.system.px_to_pos(p);
                        if shift_pressed {
                            if editor_state.has_selections() {
                                is_selecting = editor_state.extend_selection_to(pos);
                            } else {
                                is_selecting = Some(editor_state.set_single_caret(pos));
                            }
                        } else if alt_pressed {
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
                    let position: LogicalPosition<f32> = position.to_logical(render.system.scale_factor.into());
                    let p = (position.x as f32, position.y as f32);
                    mouse_at = Some(p);

                    if let Some(id) = is_selecting {
                        let caret = render.system.px_to_pos(p);
                        editor_state.drag_select(caret, id);
                    }
                }
                WindowEvent::Moved(u) => {
                    println!("moved {:?}", u);
                }
                // WindowEvent::DragEnter { paths, position } => {
                    // println!("drag enter {:?}", position);
                    // for path in paths {
                    //     println!("  - {:?}", path);
                    // }
                // }
                WindowEvent::DragOver { position } => {
                    let position: LogicalPosition<f32> = position.to_logical(render.system.scale_factor.into());
                    let pos = render.system.px_to_pos((position.x as f32, position.y as f32));

                    editor_state.file_drag_hover(pos);
                }
                WindowEvent::DragDrop { mut paths, position } => {
                    let Some(filepath) = paths.pop() else {
                        return;
                    };

                    let position: LogicalPosition<f32> = position.to_logical(render.system.scale_factor.into());
                    let pos = render.system.px_to_pos((position.x as f32, position.y as f32));

                    editor_state.insert(pos, Token::Widget { id: 0, width: 5 }.into(), true);
                    let widget = SampleWidget::new(filepath);
                    let id = widgets.len();
                    let width = widget.column_width();
                    widgets.push(Box::new(widget));

                    editor_state.insert(pos, Token::Widget { id, width }.into(), true);
                }
                _ => (),
            },
            winit::event::Event::RedrawRequested(_) => {
                render.draw(&editor_state);
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
