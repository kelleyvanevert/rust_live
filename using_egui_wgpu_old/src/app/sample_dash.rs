use std::{cell::RefCell, time::Instant};

use egui::{epaint::*, *};

use crate::read_audio_file::{read_audio_file, AudioTrackInfo};

use super::dash::{Dash, DASH_HEIGHT};

pub struct SampleDash {
    audio_file: AudioTrackInfo,
    width: usize,
    summary: RefCell<Option<Summary>>,
}

struct Summary {
    overall_max: f32,
    samples_overview: Vec<(f32, f32, f32)>,
}

impl SampleDash {
    pub fn new(filepath: &str) -> Self {
        let width = 0;
        let audio_file = read_audio_file(filepath);

        Self {
            audio_file,
            width,
            summary: RefCell::new(None),
        }
    }
}

impl Dash for SampleDash {
    fn ui(&mut self, ui: &mut Ui) {
        let (response, painter) =
            ui.allocate_painter(vec2(f32::INFINITY, DASH_HEIGHT), Sense::click());

        let mut rect = response.rect;

        rect.max.x = ui.clip_rect().max.x;

        if !ui.is_rect_visible(rect) {
            return;
        }

        painter.rect_filled(rect, 0.0, self.bg_color());

        ui.allocate_ui_at_rect(
            Rect {
                min: rect.left_top() + vec2(20.0, 17.0),
                max: rect.left_top() + vec2(f32::INFINITY, 40.0),
            },
            |ui| {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.label(
                        RichText::new(self.title())
                            .size(18.0)
                            .family(FontFamily::Name("Bold".into()))
                            .color(self.title_color()),
                    );

                    ui.horizontal_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("Length: 2.3s")
                                .color(hex_color!("#ffffff66"))
                                .size(12.0),
                        );

                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("Stereo")
                                .color(hex_color!("#ffffff66"))
                                .size(12.0),
                        );
                    });
                });
            },
        );

        let sample_rect = Rect {
            min: rect.min + vec2(20.0, 50.0),
            max: rect.max - vec2(20.0, 30.0),
        };

        // // debug
        // ui.painter()
        //     .rect_filled(sample_rect, 0.0, hex_color!("#cc000077"));

        //         ui.ctx().request_repaint();

        let width = sample_rect.width() as usize / 2;
        if width != self.width {
            self.width = width;

            println!("update");
            let t0 = Instant::now();

            let num_samples = self.audio_file.samples.len();
            // physical pixels, btw
            let samples_per_pixel = num_samples / width;

            // (min, max, rms)
            let mut samples_overview: Vec<(f32, f32, f32)> = vec![];

            let (mut overall_min, mut overall_max) = (0.0, 0.0);
            let (mut min, mut max) = (0.0, 0.0);

            let mut count = 0;
            let mut rms_range = vec![];

            fn calculate_rms(samples: &Vec<f32>) -> f32 {
                let sqr_sum = samples.iter().fold(0.0, |sqr_sum, s| {
                    let sample = *s as f32;
                    sqr_sum + sample * sample
                });

                (sqr_sum / samples.len() as f32).sqrt()
            }

            for i in 0..num_samples {
                let sample = self.audio_file.samples[i];
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

            let _ = self.summary.borrow_mut().insert(Summary {
                overall_max: overall_max.max(-overall_min),
                samples_overview,
            });
        }

        let mut shapes = vec![];

        // shapes.push(egui::epaint::Shape::Rect(egui::epaint::RectShape {
        //     rect: paint_rect,
        //     rounding: 0.0.into(),
        //     fill: hex_color!("#00000055"),
        //     stroke: Stroke::NONE,
        // }));

        let summary = self.summary.borrow();
        let summary = summary.as_ref().unwrap();

        let height = sample_rect.height();
        let half = height / 2.0;
        let scale = 0.85 * half * (1.0 / summary.overall_max);
        let x0 = sample_rect.min.x;
        let y0 = sample_rect.min.y + half;

        for (i, &(min, max, rms)) in summary.samples_overview.iter().enumerate() {
            let x = 2.0 * i as f32 + x0;
            shapes.push(Shape::line_segment(
                [pos2(x, y0 + min * scale), pos2(x, y0 + max * scale)],
                Stroke::new(1.2, hex_color!("#ffffff77")),
            ));
            shapes.push(Shape::line_segment(
                [pos2(x, y0 - rms * scale), pos2(x, y0 + rms * scale)],
                Stroke::new(2.0, hex_color!("#ffffffff")),
            ));
        }

        ui.painter().extend(shapes);
    }

    fn title(&self) -> String {
        "Sample".into()
    }

    fn title_color(&self) -> Color32 {
        hex_color!("#ffffff")
    }

    fn bg_color(&self) -> Color32 {
        hex_color!("#0B07C7")
    }
}
