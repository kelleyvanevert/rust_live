use rodio::{OutputStream, Source};
use std::f32::consts::TAU;

#[allow(unused)]
pub fn sines(secs: u64) {
    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);

    for n in 0..wave_table_size {
        wave_table.push((TAU * n as f32 / wave_table_size as f32).sin());
    }

    let osc = WaveTableOsc::sine(440.0);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let osc = osc
        .mix(WaveTableOsc::sine(440.0 * 0.1).amplify(1.0))
        .mix(WaveTableOsc::sine(440.0 * 0.1).amplify(1.0))
        .mix(WaveTableOsc::sine(440.0 * 0.2).amplify(1.0))
        .mix(WaveTableOsc::sine(440.0 * 0.3).amplify(1.0))
        .mix(WaveTableOsc::sine(440.0 * 1.0).amplify(0.7))
        .mix(WaveTableOsc::sine(440.0 * 2.0).amplify(0.2));

    let _res = stream_handle.play_raw(osc.convert_samples());

    std::thread::sleep(std::time::Duration::from_secs(secs));
}

struct WaveTableOsc {
    wave_table: Vec<f32>,
    index: f32,
    index_increment: f32,
}

impl WaveTableOsc {
    const SAMPLE_RATE: u32 = 441_000;

    fn new(wave_table: Vec<f32>) -> Self {
        Self {
            wave_table,
            index: 0.0,
            index_increment: 0.0,
        }
    }

    fn sine(frequency_hz: f32) -> Self {
        let wave_table_size = 64;
        let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);

        for n in 0..wave_table_size {
            wave_table.push((TAU * n as f32 / wave_table_size as f32).sin());
        }

        let mut osc = WaveTableOsc::new(wave_table);

        osc.set_frequency(frequency_hz);

        osc
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * self.wave_table.len() as f32 / Self::SAMPLE_RATE as f32;
    }

    fn get_next_sample(&mut self) -> f32 {
        let sample = self.get_sample(self.index);
        self.index = (self.index + self.index_increment) % self.wave_table.len() as f32;

        sample
    }

    fn get_sample(&self, index: f32) -> f32 {
        let index_trunc = index as usize;
        let index_next = (index_trunc + 1) % self.wave_table.len();

        let index_next_weight = self.index - (index_trunc as f32);
        let index_trunc_weight = 1.0 - index_next_weight;

        index_trunc_weight * self.wave_table[index_trunc]
            + index_next_weight * self.wave_table[index_next]
    }
}

impl Iterator for WaveTableOsc {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.get_next_sample());
    }
}

impl Source for WaveTableOsc {
    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        Self::SAMPLE_RATE
    }

    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
