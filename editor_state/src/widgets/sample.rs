use std::path::PathBuf;

use super::Widget;

pub struct SampleWidget {
    pub filepath: PathBuf,
}

impl Widget for SampleWidget {
    fn width_in_editor(&self) -> usize {
        5
    }
}
