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

    fn draw(&self, frame: &mut [u8], _width: usize, _height: usize) {
        let c = if self.hovering { 0x9a } else { 0xf0 };

        for pixel in frame.chunks_exact_mut(4) {
            pixel[0] = c; // R
            pixel[1] = c; // G
            pixel[2] = c; // B
            pixel[3] = 0xff; // A
        }
    }
}
