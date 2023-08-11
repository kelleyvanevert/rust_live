use rodio::Source;
use std::{
    f32::consts::TAU,
    sync::mpsc::{self, Receiver, Sender},
};

pub enum Event {
    SetFrequency(f32),
    SetVolume(f32),
    SetSquareness(f32),
}

pub struct Osc {
    sample_rate: u64,
    volume: f32,
    frequency: f32,
    squareness: f32,
    rad: f32,
    front: (Sender<Event>, Receiver<Event>),
}

impl Osc {
    pub fn sine(frequency: f32, squareness: f32) -> Self {
        let front = mpsc::channel();

        Self {
            sample_rate: 44_100,
            volume: 0.5,
            frequency,
            squareness,
            rad: 0.0,
            front,
        }
    }

    fn get_next_sample(&mut self) -> f32 {
        while let Ok(ev) = self.front.1.try_recv() {
            match ev {
                Event::SetFrequency(frequency) => self.frequency = frequency,
                Event::SetVolume(volume) => self.volume = volume,
                Event::SetSquareness(squareness) => self.squareness = squareness,
                _ => {}
            }
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
        smooth_sq * self.volume
    }

    pub fn frontend(&self) -> Sender<Event> {
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
