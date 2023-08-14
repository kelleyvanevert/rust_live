pub fn ease_cubic_in_out(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x.powi(3)
    } else {
        1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
    }
}
