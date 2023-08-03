use live_editor_state::WidgetInfo;

pub trait Widget {
    fn kind(&self) -> &'static str;

    // Decide how big it should be in the code editor (only called once)
    fn column_width(&self) -> usize {
        5
    }

    // Receive events such as: suspend, update how many instances are used, mouse input stuff, etc.
    fn event(&mut self, _event: WidgetEvent) {}

    // Draw to pixel frame
    fn draw(&self, _frame: &mut [u8], _width: usize, _height: usize) {}

    // When the file is saved in "bundled" mode, this method is called
    fn bundle_resources(&self) {}

    // For debugging, or for "save as text file"
    fn describe(&self) -> String {
        format!("[no description]")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetEvent {
    Hover { uv: (f32, f32) },
    Unhover,
}

pub struct WidgetManager {
    widgets: Vec<Box<dyn Widget>>,
}

impl WidgetManager {
    pub fn new() -> Self {
        Self { widgets: vec![] }
    }

    pub fn add(&mut self, widget: Box<dyn Widget>) -> WidgetInfo {
        let id = self.widgets.len();

        let width = widget.column_width();
        let kind = widget.kind();

        self.widgets.push(widget);

        WidgetInfo { kind, id, width }
    }

    pub fn draw(&mut self, id: usize, frame: &mut [u8], width: usize, height: usize) {
        if let Some(widget) = self.widgets.get_mut(id) {
            widget.draw(frame, width, height);
        }
    }

    pub fn hover(&mut self, id: usize, uv: (f32, f32)) {
        if let Some(widget) = self.widgets.get_mut(id) {
            widget.event(WidgetEvent::Hover { uv });
        }
    }

    pub fn unhover(&mut self, id: usize) {
        if let Some(widget) = self.widgets.get_mut(id) {
            widget.event(WidgetEvent::Unhover);
        }
    }
}
