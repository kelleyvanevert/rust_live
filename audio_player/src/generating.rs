use rodio::Source;
use std::{
    f32::consts::{PI, TAU},
    sync::mpsc::{self, Receiver, Sender},
};

pub struct Osc {
    sample_rate: u64,
    frequency: f32,
    triangleness: f32,
    rad: f32,

    target_frequency: f32,
    target_triangleness: f32,

    front: (Sender<(f32, f32)>, Receiver<(f32, f32)>),
}

impl Osc {
    pub fn sine(frequency: f32, triangleness: f32) -> Self {
        let front = mpsc::channel();

        Self {
            sample_rate: 441_000,
            frequency,
            triangleness,
            rad: 0.0,

            target_frequency: frequency,
            target_triangleness: triangleness,

            front,
        }
    }

    fn get_next_sample(&mut self) -> f32 {
        if let Ok(target) = self.front.1.try_recv() {
            (self.frequency, self.triangleness) = target;
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

        // as a triangle
        let x = (self.rad + PI / 2.0) / TAU;
        let tri = 4.0 * (x - (x + 0.5).floor()).abs() - 1.0;

        (tri * self.triangleness) + (sin * (1.0 - self.triangleness))
    }

    pub fn get_freq_sender(&self) -> Sender<(f32, f32)> {
        self.front.0.clone()
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
