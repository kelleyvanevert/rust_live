use rodio::Source;
use std::{
    f32::consts::{PI, TAU},
    sync::mpsc::{self, Receiver, Sender},
};

pub struct Osc {
    sample_rate: u64,
    frequency: f32,
    squareness: f32,
    rad: f32,

    target_frequency: f32,
    target_squareness: f32,

    front: (Sender<(f32, f32)>, Receiver<(f32, f32)>),
}

impl Osc {
    pub fn sine(frequency: f32, squareness: f32) -> Self {
        let front = mpsc::channel();

        Self {
            sample_rate: 100_000,
            frequency,
            squareness,
            rad: 0.0,

            target_frequency: frequency,
            target_squareness: squareness,

            front,
        }
    }

    fn get_next_sample(&mut self) -> f32 {
        if let Ok(target) = self.front.1.try_recv() {
            (self.target_frequency, self.squareness) = target;
        }

        let diff = self.target_frequency - self.frequency;
        if diff.abs() > 0.01 {
            // "smoothing" XD
            self.frequency += 100.0 * diff / (self.sample_rate as f32);
        }

        self.rad += self.frequency * (TAU / self.sample_rate as f32);
        self.rad %= TAU;

        // as a sine
        let sin = self.rad.sin();

        // // as a triangle
        // let x = (self.rad + PI / 2.0) / TAU;
        // let tri = 4.0 * (x - (x + 0.5).floor()).abs() - 1.0;

        // // as a square
        // let sq = sin.signum();

        // as a smoothed square
        let d = 1.0 - ease_cubic_in_out(0.3 + 0.6 * self.squareness); // between 0 and 1
        let smooth_sq: f32 = fast_math::atan(sin / d) / fast_math::atan(1.0 / d);

        // sin
        smooth_sq
    }

    pub fn get_freq_sender(&self) -> Sender<(f32, f32)> {
        self.front.0.clone()
    }
}

fn ease_cubic_in_out(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x.powi(3)
    } else {
        1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
    }
}

impl Iterator for Osc {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.get_next_sample());
    }
}

impl Source for Osc {
    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate as u32
    }

    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
