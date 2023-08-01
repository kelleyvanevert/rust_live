use live_editor_state::EditorState;
use nannou::{prelude::*, winit::event::VirtualKeyCode};

fn main() {
    nannou::app(model).update(update).run();
}

// Model represents the state of our application. We don't have any state in this demonstration, so
// for now it is just an empty struct.
struct Model {
    size: Vec2,
    window_id: WindowId,
    editor_state: EditorState,
}

fn model(app: &App) -> Model {
    let size = vec2(800.0, 500.0);

    let window_id = app
        .new_window()
        .size(size.x as u32, size.y as u32)
        .event(event)
        .view(view)
        .build()
        .unwrap();

    Model {
        size,
        window_id,
        editor_state: EditorState::new(),
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn event(app: &App, model: &mut Model, event: WindowEvent) {
    println!("event {:?}", event);

    match event {
        // Keyboard events
        KeyPressed(key) => match key {
            VirtualKeyCode::Escape => app.quit(),
            _ => {}
        },
        KeyReleased(_key) => {}
        ReceivedCharacter(_char) => {}

        // Mouse events
        MouseMoved(_pos) => {}
        MousePressed(_button) => {}
        MouseReleased(_button) => {}
        MouseWheel(_amount, _phase) => {}
        MouseEntered => {}
        MouseExited => {}

        // Touch events
        Touch(_touch) => {}
        TouchPressure(_pressure) => {}

        // Window events
        Moved(_pos) => {}
        Resized(size) => {
            model.size = size;
            // let window = app.window(model.window_id).unwrap();
        }
        HoveredFile(_path) => {}
        DroppedFile(_path) => {}
        HoveredFileCancelled => {}
        Focused => {}
        Unfocused => {}
        Closed => {}
    }
}

// Put your drawing code, called once per frame, per window.
fn view(_app: &App, _model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);
}
