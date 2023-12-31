use creak;
use rfd::FileDialog;
use std::{cell::RefCell, time::Instant};

use crate::{render::WidgetTexture, ui::WidgetEvent, widget::Widget};

struct Theme {
    background: [u8; 4],
    wave: [u8; 4],
    rms: [u8; 4],
    line: [u8; 4],
}

struct Summary {
    overall_max: f32,
    samples_overview: Vec<(f32, f32, f32)>,
}

pub struct SampleWidget {
    filepath: Option<String>,
    selected: bool,
    hovering: Option<f32>, // x within widget
    samples: Option<Vec<f32>>,
    summary: RefCell<Option<Summary>>,
}

impl SampleWidget {
    pub fn new(filepath: impl Into<String>) -> Self {
        let mut widget = Self {
            filepath: None,
            selected: false,
            hovering: None,
            samples: None,
            summary: RefCell::new(None),
        };

        widget.read(filepath.into());

        widget
    }

    fn read(&mut self, filepath: String) -> bool {
        let decoder = creak::Decoder::open(&filepath).ok();

        let Some(decoder) = decoder else {
            println!("Could not read audio file at: {:?}", filepath);
            return false;
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
            self.filepath = Some(filepath);
            true
        } else {
            println!("  error reading samples :(");
            false
        }
    }
}

impl Widget for SampleWidget {
    fn kind(&self) -> &'static str {
        "sample"
    }

    fn column_width(&self) -> usize {
        6
    }

    fn event(&mut self, event: WidgetEvent) -> bool {
        match event {
            WidgetEvent::Hover { bounds, mouse } => {
                // `bounds` and `mouse` are logical pixels, but we draw in physical pixels
                //  .. so (hacky) just go ahead and multiply by 2 for now
                self.hovering = Some((mouse.0 - bounds.0) * 2.0)
            }
            WidgetEvent::Unhover => self.hovering = None,
            WidgetEvent::MouseDown { .. } => {
                self.selected = true;
            }
            WidgetEvent::Press { double, .. } => {
                if double && let Some(filepath) = FileDialog::new()
                    .add_filter("audio", &["wav", "mp3", "ogg", "flac"])
                    // .set_directory("~")
                    .pick_file()
                {
                    let filepath = filepath.as_path().to_str().unwrap();
                    self.read(filepath.into());
                }

                return false;
            }
            WidgetEvent::Release { double } => {
                if !double {
                    self.selected = false;
                }
            }
            _ => {}
        }

        false
    }

    fn draw(&self, frame: &mut WidgetTexture) {
        // physical pixels, btw
        let width = frame.width();
        let height = frame.height();

        let Some(samples) = &self.samples else {
            frame.clear(&[0xff, 0x00, 0x00, 0xff]);
            return;
        };

        let mut summary = self.summary.borrow_mut();
        let summary = summary.get_or_insert_with(|| {
            let t0 = Instant::now();

            let num_samples = samples.len();
            // physical pixels, btw
            let samples_per_pixel = num_samples / (width - 4);

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
                samples_overview,
            }
        });

        let theme = if self.selected {
            Theme {
                background: [0x00, 0x00, 0x00, 0xff],
                wave: [0xaa, 0xaa, 0xaa, 0xff],
                rms: [0xe5, 0xe5, 0xe5, 0xff],
                line: [0xe5, 0xe5, 0xe5, 0xff],
            }
        } else {
            Theme {
                background: [0xe5, 0xe5, 0xe5, 0xff],
                wave: [0x99, 0x99, 0x99, 0xff],
                rms: [0x00, 0x00, 0x00, 0xff],
                line: [0x00, 0x00, 0x00, 0xff],
            }
        };

        frame.clear(&theme.background);

        let half = (height as f32) / 2.0;
        let scale = 0.85 * half * (1.0 / summary.overall_max);

        for x in 2..(width - 4) {
            let (min, max, rms) = summary.samples_overview[x];

            let ymin = (min * scale + half).round() as usize;
            let ymax = (max * scale + half).round() as usize;
            for y in ymin..ymax {
                frame.set_pixel(x, y, &theme.wave);
            }

            let ymin = (-rms * scale + half).round() as usize;
            let ymax = (rms * scale + half).round() as usize;
            for y in ymin..ymax {
                frame.set_pixel(x, y, &theme.rms);
            }
        }

        if let Some(x) = self.hovering {
            let x = (x.round() as usize).max(0).min(width - 2);
            for y in 0..height {
                frame.set_pixel(x, y, &theme.line);
            }
        }

        let empty: [u8; 4] = [0, 0, 0, 0];

        // top left
        frame.set_pixel(0, 0, &empty);
        frame.set_pixel(1, 0, &empty);
        frame.set_pixel(2, 0, &empty);
        frame.set_pixel(0, 1, &empty);
        frame.set_pixel(1, 1, &empty);
        frame.set_pixel(0, 2, &empty);

        // top right
        frame.set_pixel(width - 1 - 0, 0, &empty);
        frame.set_pixel(width - 1 - 1, 0, &empty);
        frame.set_pixel(width - 1 - 2, 0, &empty);
        frame.set_pixel(width - 1 - 0, 1, &empty);
        frame.set_pixel(width - 1 - 1, 1, &empty);
        frame.set_pixel(width - 1 - 0, 2, &empty);

        // bottom left
        frame.set_pixel(0, height - 1 - 0, &empty);
        frame.set_pixel(1, height - 1 - 0, &empty);
        frame.set_pixel(2, height - 1 - 0, &empty);
        frame.set_pixel(0, height - 1 - 1, &empty);
        frame.set_pixel(1, height - 1 - 1, &empty);
        frame.set_pixel(0, height - 1 - 2, &empty);

        // bottom right
        frame.set_pixel(width - 1 - 0, height - 1 - 0, &empty);
        frame.set_pixel(width - 1 - 1, height - 1 - 0, &empty);
        frame.set_pixel(width - 1 - 2, height - 1 - 0, &empty);
        frame.set_pixel(width - 1 - 0, height - 1 - 1, &empty);
        frame.set_pixel(width - 1 - 1, height - 1 - 1, &empty);
        frame.set_pixel(width - 1 - 0, height - 1 - 2, &empty);
    }
}

fn calculate_rms(samples: &Vec<f32>) -> f32 {
    let sqr_sum = samples.iter().fold(0.0, |sqr_sum, s| {
        let sample = *s as f32;
        sqr_sum + sample * sample
    });

    (sqr_sum / samples.len() as f32).sqrt()
}
