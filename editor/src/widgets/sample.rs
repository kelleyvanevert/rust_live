use std::path::PathBuf;

use crate::widget::{Widget, WidgetEvent};

pub struct SampleWidget {
    filepath: PathBuf,
    hovering: bool,
}

impl SampleWidget {
    pub fn new(filepath: PathBuf) -> Self {
        Self {
            filepath,
            hovering: false,
        }
    }
}

impl Widget for SampleWidget {
    fn column_width(&self) -> usize {
        self.filepath.to_str().unwrap().len().min(7)
    }

    fn event(&mut self, event: WidgetEvent) {
        match event {
            WidgetEvent::Hover => self.hovering = true,
            WidgetEvent::Unhover => self.hovering = false,
        }
    }

    fn draw(&self, _frame: &mut [u8], _width: f32, _height: f32) {
        // TODO
    }
}
