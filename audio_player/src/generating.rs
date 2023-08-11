use rodio::Source;
use std::{
    f32::consts::TAU,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::util::ease_cubic_in_out;

#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub volume: Option<f32>,
    pub frequency: Option<f32>,
    pub squareness: Option<f32>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            volume: None,
            frequency: None,
            squareness: None,
        }
    }
}

impl Params {
    pub fn freq(mut self, frequency: f32) -> Self {
        self.frequency = Some(frequency);
        self
    }

    pub fn vol(mut self, volume: f32) -> Self {
        self.volume = Some(volume);
        self
    }

    pub fn sq(mut self, squareness: f32) -> Self {
        self.squareness = Some(squareness);
        self
    }
}

pub fn silent() -> Params {
    Params::default().vol(0.0)
}

impl From<f32> for Params {
    fn from(frequency: f32) -> Self {
        Self::default().freq(frequency)
    }
}

pub fn lerp_params(x: f32, a: Params, b: Params) -> Params {
    Params {
        volume: lerp_option(x, a.volume, b.volume),
        frequency: lerp_option(x, a.frequency, b.frequency),
        squareness: lerp_option(x, a.squareness, b.squareness),
    }
}

fn lerp_option(x: f32, a: Option<f32>, b: Option<f32>) -> Option<f32> {
    match (a, b) {
        (None, None) => None,
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (Some(a), Some(b)) => Some(a + x * (b - a)),
    }
}

const SAMPLE_RATE: u32 = 44_100;

pub struct Osc {
    volume: f32,
    frequency: f32,
    squareness: f32,
    rad: f32,
    front: (Sender<Params>, Receiver<Params>),
}

impl Osc {
    pub fn sine<P: Into<Params>>(params: P) -> Self {
        let front = mpsc::channel();

        let params: Params = params.into();
        let volume = params.volume.unwrap_or(0.5);
        let frequency = params.frequency.unwrap_or(440.0);
        let squareness = params.squareness.unwrap_or(0.5);

        Self {
            volume,
            frequency,
            squareness,
            rad: 0.0,
            front,
        }
    }

    fn apply(&mut self, params: Params) {
        if let Some(volume) = params.volume {
            self.volume = volume;
        }
        if let Some(frequency) = params.frequency {
            self.frequency = frequency;
        }
        if let Some(squareness) = params.squareness {
            self.squareness = squareness;
        }
    }

    fn get_next_sample(&mut self) -> f32 {
        while let Ok(params) = self.front.1.try_recv() {
            self.apply(params);
        }

        self.rad += self.frequency * (TAU / SAMPLE_RATE as f32);
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
        smooth_sq * self.volume
    }

    pub fn frontend(&self) -> Sender<Params> {
        self.front.0.clone()
    }
}

impl Iterator for Osc {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_next_sample())
    }
}

impl Source for Osc {
    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

pub struct Mix {
    inputs: Vec<Osc>,
}

impl Mix {
    pub fn new(inputs: Vec<Osc>) -> Self {
        Self { inputs }
    }
}

impl Iterator for Mix {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inputs.iter_mut().map(|s| s.next().unwrap()).sum())
    }
}

impl Source for Mix {
    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
