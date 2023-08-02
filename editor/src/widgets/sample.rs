use std::path::PathBuf;

use crate::widget::Widget;

pub struct SampleWidget {
    filepath: PathBuf,
}

impl SampleWidget {
    pub fn new(filepath: PathBuf) -> Self {
        Self { filepath }
    }
}

impl Widget for SampleWidget {
    fn column_width(&self) -> usize {
        self.filepath.to_str().unwrap().len().min(7)
    }
}
