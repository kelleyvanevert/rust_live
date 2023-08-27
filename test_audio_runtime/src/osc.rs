use itertools::Itertools;
use std::{
    collections::HashMap,
    f32::consts::TAU,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{read_audio_file::read_audio_file, util::ease_cubic_in_out};

const SAMPLE_RATE: u32 = 44_100;

pub trait AudioNode {
    fn parameters(&self) -> Vec<String>;
    fn named_parameters(&self) -> Vec<String>;
    fn map(&mut self, name: String, parameter: String);
    fn apply(&mut self, param: String, value: f32);
    fn get_next_sample(&self) -> f32;
    fn tick(&mut self);
}

pub struct Osc {
    // parameters
    volume: f32,
    frequency: f32,
    squareness: f32,

    // audio node helper stuff
    named_parameters: HashMap<String, String>,

    // state
    rad: f32,
}

impl Default for Osc {
    fn default() -> Self {
        Self {
            volume: 0.3,
            frequency: 440.0,
            squareness: 0.3,
            named_parameters: HashMap::new(),
            rad: 0.0,
        }
    }
}

impl AudioNode for Osc {
    fn named_parameters(&self) -> Vec<String> {
        self.named_parameters.keys().cloned().collect_vec()
    }

    fn map(&mut self, name: String, parameter: String) {
        self.named_parameters.insert(name, parameter);
    }

    fn parameters(&self) -> Vec<String> {
        vec!["volume".into(), "frequency".into(), "squareness".into()]
    }

    // TODO how to do broadcasting of other (collection) types of values?
    fn apply(&mut self, mut param: String, value: f32) {
        if let Some(actual) = self.named_parameters.get(&param) {
            param = actual.clone();
        }

        match &param as &str {
            "volume" => self.volume = value,
            "frequency" => self.frequency = value,
            "squareness" => self.squareness = value,
            _ => {}
        }
    }

    fn tick(&mut self) {
        self.rad += self.frequency * (TAU / SAMPLE_RATE as f32);
        self.rad %= TAU;
    }

    fn get_next_sample(&self) -> f32 {
        // while let Ok(params) = self.front.1.try_recv() {
        //     self.apply(params);
        // }

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
}

pub struct Sine {
    osc: Osc,
}

impl Default for Sine {
    fn default() -> Self {
        let mut osc = Osc::default();
        osc.apply("squareness".into(), 0.0);

        Self { osc }
    }
}

impl AudioNode for Sine {
    fn parameters(&self) -> Vec<String> {
        self.osc
            .parameters()
            .into_iter()
            .filter(|p| p != "squareness")
            .collect()
    }

    fn named_parameters(&self) -> Vec<String> {
        self.osc.named_parameters()
    }

    fn map(&mut self, name: String, parameter: String) {
        self.osc.map(name, parameter);
    }

    fn apply(&mut self, param: String, value: f32) {
        self.osc.apply(param, value);
    }

    fn tick(&mut self) {
        self.osc.tick();
    }

    fn get_next_sample(&self) -> f32 {
        self.osc.get_next_sample()
    }
}

pub struct Mix {
    inputs: Vec<Box<dyn AudioNode + Send>>,
}

impl Mix {
    pub fn add(mut self, node: Box<dyn AudioNode + Send>) -> Self {
        self.inputs.push(node);
        self
    }
}

impl Default for Mix {
    fn default() -> Self {
        Mix { inputs: vec![] }
    }
}

impl AudioNode for Mix {
    fn named_parameters(&self) -> Vec<String> {
        self.inputs
            .iter()
            .flat_map(|n| n.named_parameters())
            .dedup()
            .collect::<Vec<_>>()
    }

    fn map(&mut self, _name: String, _parameter: String) {}

    fn parameters(&self) -> Vec<String> {
        vec![]
    }

    fn apply(&mut self, param: String, value: f32) {
        for n in &mut self.inputs {
            n.apply(param.clone(), value);
        }
    }

    fn tick(&mut self) {
        for input in &mut self.inputs {
            input.tick();
        }
    }

    fn get_next_sample(&self) -> f32 {
        self.inputs.iter().map(|n| n.get_next_sample()).sum()
    }
}

#[derive(Debug, Clone)]
pub struct Sample {
    samples: Vec<f32>,
    delay: usize,
    index: usize,
    attack_samples: usize,
    release_samples: usize,
    repeat: bool,

    // audio node helper stuff
    named_parameters: HashMap<String, String>,
}

impl Sample {
    pub fn new(filepath: &str) -> Self {
        let info = read_audio_file(filepath);
        let samples = info.get_mono_samples();
        Self {
            samples,
            delay: 0,
            index: 0,
            attack_samples: SAMPLE_RATE as usize / 100,
            release_samples: SAMPLE_RATE as usize / 100,
            repeat: false,
            named_parameters: HashMap::new(),
        }
    }

    pub fn delay(mut self, secs: f32) -> Self {
        self.delay += (secs * SAMPLE_RATE as f32) as usize;
        self
    }
}

impl AudioNode for Sample {
    fn parameters(&self) -> Vec<String> {
        vec!["seek".into(), "repeat".into()]
    }

    fn named_parameters(&self) -> Vec<String> {
        self.named_parameters.keys().cloned().collect_vec()
    }

    fn map(&mut self, name: String, parameter: String) {
        self.named_parameters.insert(name, parameter);
    }

    fn apply(&mut self, mut param: String, value: f32) {
        if let Some(actual) = self.named_parameters.get(&param) {
            param = actual.clone();
        }

        match &param as &str {
            "repeat" => self.repeat = value >= 0.5,
            "seek" => {
                let i = (value * self.samples.len() as f32) as usize;
                self.index = self.delay + i;
            }
            _ => {}
        }
    }

    fn tick(&mut self) {
        if self.repeat && self.index >= self.delay && self.index - self.delay >= self.samples.len()
        {
            self.index = self.delay;
        }

        // `self.start`-based
        self.index += 1;
    }

    fn get_next_sample(&self) -> f32 {
        // `self.start`-based
        let i = self.index;

        if i < self.delay || i - self.delay >= self.samples.len() {
            return 0.0;
        }

        // 0-based
        let i = i - self.delay;

        let volume = if i < self.attack_samples {
            i as f32 / self.attack_samples as f32
        } else if self.samples.len() - i < self.release_samples {
            (self.samples.len() - i) as f32 / self.release_samples as f32
        } else {
            1.0
        };

        self.samples[i] * volume
    }
}

pub struct Wrapper {
    node: Box<dyn AudioNode + Send>,
    frontend: (Sender<(String, f32)>, Receiver<(String, f32)>),
}

impl Wrapper {
    pub fn new(node: Box<dyn AudioNode + Send>) -> Self {
        let frontend = mpsc::channel();

        Self { node, frontend }
    }

    pub fn get_next_sample(&mut self) -> f32 {
        self.node.tick();

        while let Ok((name, value)) = self.frontend.1.try_recv() {
            self.node.apply(name, value);
        }

        self.node.get_next_sample()
    }

    pub fn get_frontend(&self) -> Sender<(String, f32)> {
        self.frontend.0.clone()
    }
}
