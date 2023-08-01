pub mod sample;

pub trait Widget {
    fn width_in_editor(&self) -> usize;
}
