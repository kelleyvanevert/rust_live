use rodio::{OutputStream, Source};
use std::f32::consts::TAU;

#[allow(unused)]
pub fn generating(secs: u64) {
    let osc = Osc::sine(440.0 * 0.1)
        .amplify(1.0)
        .mix(Osc::sine(440.0 * 0.1).amplify(1.0))
        .mix(Osc::sine(440.0 * 0.2).amplify(1.0))
        .mix(Osc::sine(440.0 * 0.3).amplify(1.0))
        .mix(Osc::sine(440.0 * 1.0).amplify(0.7))
        .mix(Osc::sine(440.0 * 2.0).amplify(0.2));

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let _res = stream_handle.play_raw(osc.convert_samples());

    std::thread::sleep(std::time::Duration::from_secs(secs));
}

struct Osc {
    sample_rate: u64,
    frequency: f32,
    sample: u64,
}

impl Osc {
    fn sine(frequency: f32) -> Self {
        Self {
            sample_rate: 441_000,
            frequency,
            sample: 0,
        }
    }

    fn get_next_sample(&mut self) -> f32 {
        let time = self.sample as f32 / self.sample_rate as f32;
        let rad = time * self.frequency * TAU;
        let sample = rad.sin();

        self.sample += 1;

        sample
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
