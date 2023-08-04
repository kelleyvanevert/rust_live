#![feature(let_chains)]
#![feature(slice_group_by)]

mod clipboard;
mod highlight;
mod render;
mod util;
mod widget;
mod widgets;

use clipboard::Clipboard;
use live_editor_state::{Direction, EditorState, LineData, MoveVariant, Pos, Token};
use render::Renderer;
use std::time::{Duration, Instant, SystemTime};
use widget::{WidgetEvent, WidgetManager};
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

struct Context {
    mouse_at: Option<(f32, f32)>,
    shift_pressed: bool,
    alt_pressed: bool,
    meta_or_ctrl_pressed: bool,
}

impl Context {
    fn new() -> Self {
        Self {
            mouse_at: None,
            shift_pressed: false,
            alt_pressed: false,
            meta_or_ctrl_pressed: false,
        }
    }
}

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

    let mut renderer = pollster::block_on(render::Renderer::new(&window));

    let mut editor = Editor::new();
    let mut ctx = Context::new();

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
                        // *control_flow = ControlFlow::Exit;
                        editor.editor_state.deselect();
                    }
                    // (Key::GoBack, ElementState::Pressed) if !code_section.text.is_empty() => {
                    //     let mut end_text = code_section.text.remove(code_section.text.len() - 1);
                    //     end_text.text.pop();
                    //     if !end_text.text.is_empty() {
                    //         code_section.text.push(end_text);
                    //     }
                    // }
                    (Key::Tab, ElementState::Pressed) => {
                        if ctx.shift_pressed {
                            editor.editor_state.untab();
                        } else {
                            editor.editor_state.tab();
                        }
                    }
                    (Key::Space, ElementState::Pressed) => {
                        editor.editor_state.write(" ");
                    }
                    (Key::Enter, ElementState::Pressed) => {
                        editor.editor_state.write("\n");
                    }
                    (Key::Backspace, ElementState::Pressed) => {
                        editor.editor_state.backspace(if ctx.alt_pressed {
                            MoveVariant::ByWord
                        } else if ctx.meta_or_ctrl_pressed {
                            MoveVariant::UntilEnd
                        } else {
                            MoveVariant::ByToken
                        });
                    }
                    (Key::ArrowUp | Key::ArrowDown, ElementState::Pressed)
                        if ctx.meta_or_ctrl_pressed && ctx.alt_pressed =>
                    {
                        editor
                            .editor_state
                            .add_caret_vertically(match logical_key.clone() {
                                Key::ArrowUp => Direction::Up,
                                Key::ArrowDown => Direction::Down,
                                _ => unreachable!(),
                            });
                    }
                    (
                        Key::ArrowUp | Key::ArrowRight | Key::ArrowDown | Key::ArrowLeft,
                        ElementState::Pressed,
                    ) => {
                        editor.editor_state.move_caret(
                            match logical_key.clone() {
                                Key::ArrowUp => Direction::Up,
                                Key::ArrowRight => Direction::Right,
                                Key::ArrowDown => Direction::Down,
                                Key::ArrowLeft => Direction::Left,
                                _ => unreachable!(),
                            },
                            ctx.shift_pressed,
                            if ctx.alt_pressed {
                                MoveVariant::ByWord
                            } else if ctx.meta_or_ctrl_pressed {
                                MoveVariant::UntilEnd
                            } else {
                                MoveVariant::ByToken
                            },
                        );
                    }
                    (Key::Character(s), ElementState::Pressed) => {
                        if s.as_str() == "c" && ctx.meta_or_ctrl_pressed {
                            // todo improve (ctrl/meta depending on OS)
                            editor.clipboard.write(editor.editor_state.copy());
                        } else if s.as_str() == "x" && ctx.meta_or_ctrl_pressed {
                            // todo improve (ctrl/meta depending on OS)
                            editor.clipboard.write(editor.editor_state.cut());
                        } else if s.as_str() == "v" && ctx.meta_or_ctrl_pressed {
                            // todo improve (ctrl/meta depending on OS)
                            if let Some(data) = editor.clipboard.read() {
                                editor.editor_state.paste(data);
                            }
                        } else if s.as_str() == "d" && ctx.meta_or_ctrl_pressed {
                            // todo improve (ctrl/meta depending on OS)
                            editor.editor_state.word_select();
                        } else if s.as_str() == "a" && ctx.meta_or_ctrl_pressed {
                            editor.editor_state.select_all();
                        } else {
                            editor.editor_state.write(s.as_str());
                        }
                    }
                    (Key::Alt, ElementState::Pressed) => {
                        ctx.alt_pressed = true;
                    }
                    (Key::Alt, ElementState::Released) => {
                        ctx.alt_pressed = false;
                    }
                    (Key::Shift, ElementState::Pressed) => {
                        ctx.shift_pressed = true;
                    }
                    (Key::Shift, ElementState::Released) => {
                        ctx.shift_pressed = false;
                    }
                    (Key::Meta, ElementState::Pressed) => {
                        ctx.meta_or_ctrl_pressed = true;
                    }
                    (Key::Meta, ElementState::Released) => {
                        ctx.meta_or_ctrl_pressed = false;
                    }
                    (Key::Super, ElementState::Pressed) => {
                        ctx.meta_or_ctrl_pressed = true;
                    }
                    (Key::Super, ElementState::Released) => {
                        ctx.meta_or_ctrl_pressed = false;
                    }
                    (Key::Control, ElementState::Pressed) => {
                        ctx.meta_or_ctrl_pressed = true;
                    }
                    (Key::Control, ElementState::Released) => {
                        ctx.meta_or_ctrl_pressed = false;
                    }
                    _ => {
                        // println!("key: {:?}, state: {:?}", logical_key, state);
                    }
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    if button != MouseButton::Left {
                        return;
                    }

                    match state {
                        ElementState::Pressed => {
                            editor.on_press(&renderer, &ctx);
                        }
                        ElementState::Released => {
                            editor.on_release(&renderer, &ctx);
                        }
                    }
                }
                WindowEvent::CursorEntered { .. } => {
                    println!("cursor entered");
                }
                WindowEvent::CursorLeft { .. } => {
                    println!("cursor left");
                    ctx.mouse_at = None;
                    // is_selecting = false;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let position: LogicalPosition<f32> =
                        position.to_logical(renderer.system.scale_factor.into());
                    let p = (position.x as f32, position.y as f32);
                    ctx.mouse_at = Some(p);

                    editor.on_mouse_move(&renderer, &ctx);
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
                    let position: LogicalPosition<f32> =
                        position.to_logical(renderer.system.scale_factor.into());
                    let pos = renderer
                        .system
                        .px_to_pos((position.x as f32, position.y as f32));

                    editor.editor_state.file_drag_hover(pos);
                }
                WindowEvent::DragDrop {
                    mut paths,
                    position,
                } => {
                    let Some(filepath) = paths.pop() else {
                        return;
                    };

                    let position: LogicalPosition<f32> =
                        position.to_logical(renderer.system.scale_factor.into());
                    let pos = renderer
                        .system
                        .px_to_pos((position.x as f32, position.y as f32));

                    let filepath = filepath.as_path().to_str().unwrap();
                    let widget = SampleWidget::new(filepath);
                    let widget_info = editor.widget_manager.add(Box::new(widget));

                    editor
                        .editor_state
                        .insert(pos, Token::Widget(widget_info).into(), true);
                }
                _ => (),
            },
            winit::event::Event::RedrawRequested(_) => {
                renderer.draw(&editor.editor_state, &mut editor.widget_manager);
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

struct Editor {
    widget_manager: WidgetManager,
    editor_state: EditorState,
    clipboard: Clipboard,

    is_selecting: Option<usize>,

    // I think this is like the kind of hidden state that would be required to map an immediate mode API to a more stately underlying system, btw..
    hovering_widget_id: Option<usize>,
    pressing_widget_id: Option<usize>,
}

impl Editor {
    fn new() -> Self {
        let clipboard = Clipboard::new();

        let mut widget_manager = WidgetManager::new();

        let w0 = widget_manager.add(Box::new(SampleWidget::new(
            "./res/samples/Abroxis - Extended Oneshot 019.wav",
        )));
        let w1 = widget_manager.add(Box::new(SampleWidget::new("./res/samples/meii - Teag.wav")));

        let linedata = LineData::from(
            "def beat = [..X. .X]

def main = sample_matrix%[midi.pitch.int] * fx + beat * kick

def fx = lowpass{f = sin(4hz)} + select{, 10}

def hp = osc(440, )

def matrix = [
  , , ,
  , , ,
  , , ,
  , , ,
].map(_ *= .2s)

def kick =  *= .1s",
        )
        .with_widget_at_pos(Pos { row: 4, col: 40 }, w0)
        .with_widget_at_pos(Pos { row: 6, col: 18 }, w1);

        let editor_state = EditorState::new().with_linedata(linedata);

        Self {
            widget_manager,
            editor_state,
            clipboard,

            is_selecting: None,
            hovering_widget_id: None,
            pressing_widget_id: None,
        }
    }

    fn hovering_widget(&self, renderer: &Renderer, ctx: &Context) -> Option<(usize, (f32, f32))> {
        ctx.mouse_at.and_then(|p| {
            self.editor_state
                .find_widget_at(renderer.system.px_to_pos_f(p))
        })
    }

    fn on_mouse_move(&mut self, renderer: &Renderer, ctx: &Context) {
        if let Some(p) = ctx.mouse_at {
            let hover = if self.is_selecting.is_none() {
                self.hovering_widget(renderer, ctx)
            } else {
                None
            };

            if let Some(id) = self.hovering_widget_id && hover.map(|(id, _)| id) != self.hovering_widget_id {
                self.widget_manager.event(id, WidgetEvent::Unhover);
            }
            if let Some((id, uv)) = hover {
                self.widget_manager.event(id, WidgetEvent::Hover { uv });
            }
            self.hovering_widget_id = hover.map(|(id, _)| id);

            if let Some(id) = self.is_selecting {
                let caret = renderer.system.px_to_pos(p);
                self.editor_state.drag_select(caret, id);
            }
        }
    }

    fn on_press(&mut self, renderer: &Renderer, ctx: &Context) {
        let mut handled = false;
        let press = self.hovering_widget(renderer, ctx);

        if let Some(id) = self.pressing_widget_id && press.map(|(id, _)| id) != self.pressing_widget_id {
            self.widget_manager.event(id, WidgetEvent::Release);
        }
        if let Some((id, uv)) = press {
            handled = self.widget_manager.event(id, WidgetEvent::Press { uv });
        }
        self.pressing_widget_id = press.map(|(id, _)| id);

        if handled {
            return;
        }

        if let Some(p) = ctx.mouse_at {
            let pos = renderer.system.px_to_pos(p);
            if ctx.shift_pressed {
                if self.editor_state.has_selections() {
                    self.is_selecting = self.editor_state.extend_selection_to(pos);
                } else {
                    self.is_selecting = Some(self.editor_state.set_single_caret(pos));
                }
            } else if ctx.alt_pressed {
                self.is_selecting = Some(self.editor_state.add_caret(pos));
            } else {
                self.is_selecting = Some(self.editor_state.set_single_caret(pos));
            }
        }
    }

    fn on_release(&mut self, _renderer: &Renderer, _ctx: &Context) {
        self.is_selecting = None;
    }
}
