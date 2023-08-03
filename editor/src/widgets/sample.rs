use creak;
use std::{cell::RefCell, time::Instant};

use crate::widget::{Widget, WidgetEvent};

struct Summary {
    samples_per_pixel: usize,
    overall_max: f32,
    samples_overview: Vec<(f32, f32, f32)>,
}

pub struct SampleWidget {
    filepath: String,
    hovering: Option<(f32, f32)>,
    samples: Option<Vec<f32>>,
    summary: RefCell<Option<Summary>>,
}

impl SampleWidget {
    pub fn new(filepath: impl Into<String>) -> Self {
        let mut widget = Self {
            filepath: filepath.into(),
            hovering: None,
            samples: None,
            summary: RefCell::new(None),
        };

        widget.read();

        widget
    }

    fn read(&mut self) {
        let decoder = creak::Decoder::open(&self.filepath).ok();

        let Some(decoder) = decoder else {
            println!("Could not read audio file at: {:?}", self.filepath);
            return;
        };

        let info = decoder.info();
        println!(
            "Format: {}; Channels: {}; Sample Rate: {}Hz",
            info.format(),
            info.channels(),
            info.sample_rate()
        );

        self.samples = decoder.into_samples().ok().and_then(|iter| {
            let mut samples = vec![];

            for sample in iter {
                samples.push(sample.ok()?);
            }

            Some(samples)
        });

        if self.samples.is_some() {
            println!("  READ :)");
        } else {
            println!("  error reading samples :(");
        }
    }
}

impl Widget for SampleWidget {
    fn column_width(&self) -> usize {
        6
    }

    fn event(&mut self, event: WidgetEvent) {
        match event {
            WidgetEvent::Hover { uv } => self.hovering = Some(uv),
            WidgetEvent::Unhover => self.hovering = None,
        }
    }

    fn draw(&self, frame: &mut [u8], width: usize, height: usize) {
        let Some(samples) = &self.samples else {
            for pixel in frame.chunks_exact_mut(4) {
                pixel[0] = 0xff; // R
                pixel[1] = 0x00; // G
                pixel[2] = 0x00; // B
                pixel[3] = 0xff; // A
            }
            return;
        };

        let mut summary = self.summary.borrow_mut();
        let summary = summary.get_or_insert_with(|| {
            let t0 = Instant::now();

            let num_samples = samples.len();
            let samples_per_pixel = num_samples / width;

            // (min, max, rms)
            let mut samples_overview: Vec<(f32, f32, f32)> = vec![];

            let (mut overall_min, mut overall_max) = (0.0, 0.0);
            let (mut min, mut max) = (0.0, 0.0);

            let mut count = 0;
            let mut rms_range = vec![];

            for i in 0..num_samples {
                let sample = samples[i];
                rms_range.push(sample);

                if sample < min {
                    min = sample;
                }
                if sample > max {
                    max = sample;
                }
                if sample < overall_min {
                    overall_min = sample;
                }
                if sample > overall_max {
                    overall_max = sample;
                }

                count += 1;
                if count == samples_per_pixel {
                    let rms = calculate_rms(&rms_range);
                    // println!("[min ={} max= {}, rms = {}]", min, max, rms);
                    samples_overview.push((min, max, rms));
                    count = 0;
                    min = 0.0;
                    max = 0.0;
                    rms_range = vec![];
                }
            }

            println!("Processed samples, took: {:?}", Instant::elapsed(&t0));

            Summary {
                overall_max: overall_max.max(-overall_min),
                samples_per_pixel,
                samples_overview,
            }
        });

        for pixel in frame.chunks_exact_mut(4) {
            pixel[0] = 0xe5; // R
            pixel[1] = 0xe5; // G
            pixel[2] = 0xe5; // B
            pixel[3] = 0xff; // A
        }

        let half = (height as f32) / 2.0;
        let scale = 0.85 * half * (1.0 / summary.overall_max);

        for x in 0..width {
            let (min, max, rms) = summary.samples_overview[x];

            let ymin = (min * scale + half).round() as usize;
            let ymax = (max * scale + half).round() as usize;
            for y in ymin..ymax {
                frame[(y * width + x) * 4 + 0] = 0xaa; // R
                frame[(y * width + x) * 4 + 1] = 0xaa; // G
                frame[(y * width + x) * 4 + 2] = 0xaa; // B
                frame[(y * width + x) * 4 + 3] = 0xff; // A
            }

            let ymin = (-rms * scale + half).round() as usize;
            let ymax = (rms * scale + half).round() as usize;
            for y in ymin..ymax {
                frame[(y * width + x) * 4 + 0] = 0x00; // R
                frame[(y * width + x) * 4 + 1] = 0x00; // G
                frame[(y * width + x) * 4 + 2] = 0x00; // B
                frame[(y * width + x) * 4 + 3] = 0xff; // A
            }
        }

        if let Some(uv) = self.hovering {
            let x = (uv.0 * width as f32)
                .round()
                .max(0.0)
                .min(width as f32 - 2.0) as usize;

            for y in 0..height {
                frame[(y * width + x + 0) * 4 + 0] = 0x00; // R
                frame[(y * width + x + 0) * 4 + 1] = 0x00; // G
                frame[(y * width + x + 0) * 4 + 2] = 0x00; // B
                frame[(y * width + x + 0) * 4 + 3] = 0xff; // A
                frame[(y * width + x + 1) * 4 + 0] = 0x00; // R
                frame[(y * width + x + 1) * 4 + 1] = 0x00; // G
                frame[(y * width + x + 1) * 4 + 2] = 0x00; // B
                frame[(y * width + x + 1) * 4 + 3] = 0xff; // A
            }
        }
    }
}

fn calculate_rms(samples: &Vec<f32>) -> f32 {
    let sqr_sum = samples.iter().fold(0.0, |sqr_sum, s| {
        let sample = *s as f32;
        sqr_sum + sample * sample
    });

    (sqr_sum / samples.len() as f32).sqrt()
}
