#![feature(let_chains)]
#![feature(slice_group_by)]

mod clipboard;
mod highlight;
mod render;
mod ui;
mod util;
mod widget;
mod widgets;

use clipboard::Clipboard;
use live_editor_state::{Direction, EditorState, LineData, MoveVariant, Pos, Token};
use render::Renderer;
use std::time::{Duration, Instant, SystemTime};
use ui::WidgetEvent;
use widget::WidgetManager;
use widgets::sample::SampleWidget;
use winit::dpi::{LogicalPosition, LogicalSize, Size};
use winit::event::{KeyEvent, MouseButton};
use winit::event_loop::EventLoopBuilder;
use winit::platform::macos::WindowBuilderExtMacOS;
use winit::{
    event::{ElementState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::Key,
    window::WindowBuilder,
};

struct Context {
    bounds: (f32, f32, f32, f32),
    mouse_at: Option<(f32, f32)>,
    shift: bool,
    alt: bool,
    meta_or_ctrl: bool,
}

impl Context {
    fn new(bounds: (f32, f32, f32, f32)) -> Self {
        Self {
            bounds,
            mouse_at: None,

            shift: false,
            alt: false,
            meta_or_ctrl: false,
        }
    }
}

pub fn run() {
    env_logger::init();

    let event_loop: EventLoop<WidgetEvent> = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();
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
    let mut ctx = Context::new((0.0, 0.0, renderer.width() as f32, renderer.height() as f32));

    let mut curr_press: Option<PressEventBuilder> = None;

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
                    ctx.bounds = (0.0, 0.0, renderer.width() as f32, renderer.height() as f32);
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
                        if ctx.shift {
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
                        editor.editor_state.backspace(if ctx.alt {
                            MoveVariant::ByWord
                        } else if ctx.meta_or_ctrl {
                            MoveVariant::UntilEnd
                        } else {
                            MoveVariant::ByToken
                        });
                    }
                    (Key::ArrowUp | Key::ArrowDown, ElementState::Pressed)
                        if ctx.meta_or_ctrl && ctx.alt =>
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
                            ctx.shift,
                            if ctx.alt {
                                MoveVariant::ByWord
                            } else if ctx.meta_or_ctrl {
                                MoveVariant::UntilEnd
                            } else {
                                MoveVariant::ByToken
                            },
                        );
                    }
                    (Key::Character(s), ElementState::Pressed) => {
                        if s.as_str() == "c" && ctx.meta_or_ctrl {
                            // todo improve (ctrl/meta depending on OS)
                            editor.clipboard.write(editor.editor_state.copy());
                        } else if s.as_str() == "x" && ctx.meta_or_ctrl {
                            // todo improve (ctrl/meta depending on OS)
                            editor.clipboard.write(editor.editor_state.cut());
                        } else if s.as_str() == "v" && ctx.meta_or_ctrl {
                            // todo improve (ctrl/meta depending on OS)
                            if let Some(data) = editor.clipboard.read() {
                                editor.editor_state.paste(data);
                            }
                        } else if s.as_str() == "d" && ctx.meta_or_ctrl {
                            // todo improve (ctrl/meta depending on OS)
                            editor.editor_state.word_select();
                        } else if s.as_str() == "a" && ctx.meta_or_ctrl {
                            editor.editor_state.select_all();
                        } else {
                            editor.editor_state.write(s.as_str());
                        }
                    }
                    (Key::Alt, ElementState::Pressed) => {
                        ctx.alt = true;
                    }
                    (Key::Alt, ElementState::Released) => {
                        ctx.alt = false;
                    }
                    (Key::Shift, ElementState::Pressed) => {
                        ctx.shift = true;
                    }
                    (Key::Shift, ElementState::Released) => {
                        ctx.shift = false;
                    }
                    (Key::Meta, ElementState::Pressed) => {
                        ctx.meta_or_ctrl = true;
                    }
                    (Key::Meta, ElementState::Released) => {
                        ctx.meta_or_ctrl = false;
                    }
                    (Key::Super, ElementState::Pressed) => {
                        ctx.meta_or_ctrl = true;
                    }
                    (Key::Super, ElementState::Released) => {
                        ctx.meta_or_ctrl = false;
                    }
                    (Key::Control, ElementState::Pressed) => {
                        ctx.meta_or_ctrl = true;
                    }
                    (Key::Control, ElementState::Released) => {
                        ctx.meta_or_ctrl = false;
                    }
                    _ => {
                        // println!("key: {:?}, state: {:?}", logical_key, state);
                    }
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    if let Some(mouse) = ctx.mouse_at {
                        if state == ElementState::Pressed {
                            let _ = proxy.send_event(WidgetEvent::MouseDown {
                                mouse,
                                right_click: button == MouseButton::Right,
                                bounds: ctx.bounds,
                                shift: ctx.shift,
                                alt: ctx.alt,
                                meta_or_ctrl: ctx.meta_or_ctrl,
                            });

                            if let Some(builder) = &mut curr_press && !builder.canceled_double {
                                println!("ms: {:?}", builder.started_at.elapsed().as_millis());
                                builder.has_fired = Some(true);
                                let _ = proxy.send_event(WidgetEvent::Press {
                                    double: true,
                                    mouse,
                                    right_click: button == MouseButton::Right,
                                    bounds: ctx.bounds,
                                    shift: ctx.shift,
                                    alt: ctx.alt,
                                    meta_or_ctrl: ctx.meta_or_ctrl,
                                });
                            } else {
                                curr_press = Some(PressEventBuilder::new(mouse, button == MouseButton::Right));
                            }
                        } else if state == ElementState::Released {
                            let _ = proxy.send_event(WidgetEvent::MouseUp);

                            if let Some(builder) = &mut curr_press {
                                builder.release();
                            }
                        }
                    } else {
                        println!("WEIRD 1");
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
                    let mouse = (position.x as f32, position.y as f32);
                    ctx.mouse_at = Some(mouse);

                    //(_, button, xy)
                    if let Some(builder) = &mut curr_press {
                        builder.dragged(mouse);

                        if builder.canceled_double && builder.has_fired.is_none() {
                            builder.has_fired = Some(false);
                            let _ = proxy.send_event(WidgetEvent::Press {
                                double: false,
                                mouse,
                                right_click: builder.right_click,
                                bounds: ctx.bounds,
                                shift: ctx.shift,
                                alt: ctx.alt,
                                meta_or_ctrl: ctx.meta_or_ctrl,
                            });
                        }
                    }

                    let _ = proxy.send_event(WidgetEvent::MouseMove {
                        bounds: (0.0, 0.0, renderer.width() as f32, renderer.height() as f32),
                        mouse,
                    });
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
            winit::event::Event::UserEvent(event) => {
                editor.event(&renderer, event);
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
                if let Some(mouse) = ctx.mouse_at {
                    if let Some(builder) = &mut curr_press {
                        if builder.reached_double_press_timeout() {
                            if builder.has_fired.is_none() {
                                builder.has_fired = Some(false);
                                let _ = proxy.send_event(WidgetEvent::Press {
                                    double: false,
                                    mouse,
                                    right_click: builder.right_click,
                                    bounds: ctx.bounds,
                                    shift: ctx.shift,
                                    alt: ctx.alt,
                                    meta_or_ctrl: ctx.meta_or_ctrl,
                                });
                            }

                            if let Some(double) = builder.has_fired && builder.has_released() {
                                let _ = proxy.send_event(WidgetEvent::Release { double });
                                curr_press = None;
                            }
                        }
                    }
                }

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

    fn find_widget(
        &self,
        renderer: &Renderer,
        mouse: (f32, f32),
    ) -> Option<(usize, (f32, f32, f32, f32), (f32, f32))> {
        renderer.widget_at(mouse).map(|(id, quad)| {
            return (id, quad, mouse);
        })
    }

    fn event(&mut self, renderer: &Renderer, event: WidgetEvent) -> bool {
        match event {
            WidgetEvent::Hover { .. } => {
                println!("editor:: hover");
                //
            }
            WidgetEvent::MouseMove { mouse, .. } => {
                let hover = if self.is_selecting.is_none() {
                    self.find_widget(renderer, mouse)
                } else {
                    None
                };

                if let Some(id) = self.hovering_widget_id && hover.map(|(id, _, _)| id) != self.hovering_widget_id {
                self.widget_manager.event(id, WidgetEvent::Unhover);
            }
                if let Some((id, bounds, mouse)) = hover {
                    // renderer
                    self.widget_manager
                        .event(id, WidgetEvent::Hover { bounds, mouse });
                }
                self.hovering_widget_id = hover.map(|(id, _, _)| id);

                if let Some(id) = self.is_selecting {
                    let caret = renderer.system.px_to_pos(mouse);
                    self.editor_state.drag_select(caret, id);
                }
            }
            WidgetEvent::Unhover => {
                println!("editor:: unhover");
                if let Some(id) = self.hovering_widget_id {
                    self.widget_manager.event(id, WidgetEvent::Unhover);
                }
            }
            WidgetEvent::MouseDown {
                mouse, shift, alt, ..
            } => {
                println!("editor:: mouse down");
                if let Some((id, widget_bounds, _)) = self.find_widget(renderer, mouse) {
                    self.widget_manager
                        .event(id, event.child_relative(widget_bounds));
                }

                let pos = renderer.system.px_to_pos(mouse);
                if shift {
                    if self.editor_state.has_selections() {
                        self.is_selecting = self.editor_state.extend_selection_to(pos);
                    } else {
                        self.is_selecting = Some(self.editor_state.set_single_caret(pos));
                    }
                } else if alt {
                    self.is_selecting = Some(self.editor_state.add_caret(pos));
                } else {
                    self.is_selecting = Some(self.editor_state.set_single_caret(pos));
                }
            }
            WidgetEvent::Press { double, mouse, .. } => {
                println!(
                    "editor:: press {:?}",
                    if double { "DOUBLE" } else { "single" }
                );

                // pressing widgets
                let w = self.find_widget(renderer, mouse);
                if let Some(id) = self.pressing_widget_id && w.map(|(id, _, _)| id) != self.pressing_widget_id {
                    self.widget_manager.event(id, WidgetEvent::Release { double });
                }
                if let Some((id, bounds, _)) = w {
                    self.widget_manager.event(id, event.child_relative(bounds));
                }
                self.pressing_widget_id = w.map(|(id, _, _)| id);

                // double press -> selecting words
                if double {
                    let pos = renderer.system.px_to_pos(mouse);
                    self.editor_state.select_word_at(pos);
                }
            }
            WidgetEvent::MouseUp => {
                // hmm, can't sent this to the widget w/o coords..
                println!("editor:: mouse up");
                self.is_selecting = None;
            }
            WidgetEvent::Release { .. } => {
                // hmm, can't sent this to the widget w/o coords..
                println!("editor:: release");
            }
        }

        false
    }
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((b.0 - a.0).powf(2.0) + (b.1 - a.1).powf(2.0)).sqrt()
}

const DOUBLE_PRESS_TIMEOUT_MS: u128 = 150;
const PRESS_CANCEL_DRAG_DIST: f32 = 2.0;

struct PressEventBuilder {
    started_at: Instant,
    released_at: Option<Instant>,
    canceled_double: bool,
    has_fired: Option<bool>, // false = single, true = double,

    mouse: (f32, f32),
    right_click: bool,
}

impl PressEventBuilder {
    fn new(mouse: (f32, f32), right_click: bool) -> Self {
        Self {
            started_at: Instant::now(),
            released_at: None,
            canceled_double: false,
            has_fired: None,

            mouse,
            right_click,
        }
    }

    fn dragged(&mut self, mouse: (f32, f32)) {
        if self.has_fired.is_none() && dist(self.mouse, mouse) >= PRESS_CANCEL_DRAG_DIST {
            self.canceled_double = true;
        }
    }

    fn release(&mut self) {
        self.released_at = Some(Instant::now());
    }

    fn has_released(&self) -> bool {
        self.released_at.is_some()
    }

    fn reached_double_press_timeout(&self) -> bool {
        self.started_at.elapsed().as_millis() >= DOUBLE_PRESS_TIMEOUT_MS
    }
}
