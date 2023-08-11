pub fn ease_cubic_in_out(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x.powi(3)
    } else {
        1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
    }
}

pub fn lerp(x: f32, d: (f32, f32), r: (f32, f32)) -> f32 {
    (x - d.0) / (d.1 - d.0) * (r.1 - r.0) + r.0
}
