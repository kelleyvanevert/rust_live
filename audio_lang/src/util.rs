pub fn ease_cubic_in_out(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x.powi(3)
    } else {
        1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
    }
}

pub enum Interpolation {
    Linear,
    CubicInOut,
    // others...
}

pub trait Interpolatable {
    fn interpolate(&self, rhs: &Self, x: f32) -> Self;
}

pub fn lerp<T: Interpolatable>(x: f32, a: T, b: T) -> T {
    a.interpolate(&b, x)
}
