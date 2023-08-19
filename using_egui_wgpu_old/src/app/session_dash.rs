use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};
use egui::*;
use itertools::Itertools;
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencySpectrum};

use super::dash::{Dash, DASH_HEIGHT};

const MIN_FREQ: f32 = 20.0;
const MAX_FREQ: f32 = 20_000.0;

struct RecordingInfo {
    quit: bool,
    sample_rate: u32,
    latest_samples: Vec<f32>,
    spectrum: Option<FrequencySpectrum>,
}

pub struct SessionDash {
    recording: Option<Arc<Mutex<RecordingInfo>>>,
}

impl SessionDash {
    pub fn new() -> Self {
        Self { recording: None }
    }

    fn stop_recording(&mut self) {
        if let Some(info) = &self.recording {
            let mut info = info.lock().expect("Could not lock recording info mutex");
            info.quit = true;
        }
        self.recording = None;
    }

    fn start_recording(&mut self) {
        let info = Arc::new(Mutex::new(RecordingInfo {
            quit: false,
            sample_rate: 0,
            latest_samples: vec![],
            spectrum: None,
        }));

        self.recording = Some(info.clone());

        let _handle = std::thread::spawn(move || {
            let host = cpal::default_host();
            let device = host.default_input_device().expect("No input device found");
            // println!(
            //     "Input device: {}",
            //     device.name().unwrap_or("<No name>".into())
            // );
            let config = device
                .default_input_config()
                .expect("Coult not get default input config");

            let sample_rate = config.sample_rate().0;

            info.lock().unwrap().sample_rate = sample_rate;

            // println!("Sample rate: {}", sample_rate);

            let channels = config.channels();

            let err_fn = move |err| {
                eprintln!("an error occurred on stream: {}", err);
            };

            fn write_input_data<T>(input: &[T], channels: u16, info: &Arc<Mutex<RecordingInfo>>)
            where
                T: cpal::Sample,
                f32: cpal::FromSample<T>,
            {
                let mut info = info.lock().unwrap();
                // let latest_samples: &mut Vec<f32> = info.latest_samples.as_mut();

                info.latest_samples = input
                    .chunks(channels as _)
                    .map(|frame| {
                        //
                        frame[0].to_sample::<f32>()
                    })
                    .collect::<Vec<_>>();

                if info.latest_samples.len() > 0 {
                    // apply hann window for smoothing; length must be a power of 2 for the FFT
                    // 2048 is a good starting point with 44100 kHz
                    let hann_window = hann_window(&info.latest_samples[0..]);
                    // calc spectrum
                    let spectrum_hann_window = samples_fft_to_spectrum(
                        // (windowed) samples
                        &hann_window,
                        // sampling rate
                        info.sample_rate,
                        // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
                        FrequencyLimit::Range(MIN_FREQ, MAX_FREQ),
                        // optional scale
                        Some(&divide_by_N),
                    )
                    .unwrap();

                    info.spectrum = Some(spectrum_hann_window);
                } else {
                    info.spectrum = None;
                }
            }

            let info_2 = info.clone();

            let mut stream_config: StreamConfig = config.clone().into();

            stream_config.buffer_size = BufferSize::Fixed(2048);

            let stream = match config.sample_format() {
                cpal::SampleFormat::I8 => device
                    .build_input_stream(
                        &stream_config,
                        move |data, _: &_| write_input_data::<i8>(data, channels, &info),
                        err_fn,
                        None,
                    )
                    .unwrap(),
                cpal::SampleFormat::I16 => device
                    .build_input_stream(
                        &stream_config,
                        move |data, _: &_| write_input_data::<i16>(data, channels, &info),
                        err_fn,
                        None,
                    )
                    .unwrap(),
                cpal::SampleFormat::I32 => device
                    .build_input_stream(
                        &stream_config,
                        move |data, _: &_| write_input_data::<i32>(data, channels, &info),
                        err_fn,
                        None,
                    )
                    .unwrap(),
                cpal::SampleFormat::F32 => device
                    .build_input_stream(
                        &stream_config,
                        move |data, _: &_| write_input_data::<f32>(data, channels, &info),
                        err_fn,
                        None,
                    )
                    .unwrap(),
                sample_format => {
                    panic!("Unsupported sample format '{sample_format}'")
                }
            };

            stream.play().expect("Could not start recording");

            loop {
                if info_2.lock().unwrap().quit {
                    println!("STOP STO PSTOP STOP");
                    break;
                }
            }
        });

        // self.recording = Some(info.clone());
    }
}

impl Dash for SessionDash {
    fn set_active(&mut self, active: bool) {
        if !active && self.recording.is_some() {
            self.stop_recording();
        } else if active && self.recording.is_none() {
            self.start_recording();
        }
    }

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
                });
            },
        );

        let mut specto_rect = rect.shrink2(vec2(40.0, 10.0));
        specto_rect.min.y += 50.0;
        let xmin = specto_rect.min.x;
        let xmax = specto_rect.max.x;
        let ymin = specto_rect.min.y;
        let ymax = specto_rect.max.y;

        if let Some(info) = &self.recording {
            let info = info.lock().unwrap();
            if let Some(spectrum) = &info.spectrum {
                let valmax: f32 = spectrum.max().1.val().max(0.01);

                println!("max: {:?}, len: {:?}", valmax, spectrum.data().len());

                // for (fr, fr_val) in spectrum.data().iter() {
                //     println!("{}Hz => {}", fr, fr_val)
                // }

                let line_points = spectrum
                    .data()
                    .iter()
                    .map(|&(freq, val)| {
                        let y = lin_scale(val.val(), (0.0, valmax), (ymax, ymin));
                        let x = exp_scale(
                            freq.val().log2(),
                            (MIN_FREQ.log2(), MAX_FREQ.log2()),
                            (xmin, xmax),
                        );
                        pos2(x, y)
                    })
                    .collect_vec();

                painter.add(Shape::line(
                    line_points,
                    Stroke::new(5.0, hex_color!("#ffffff")),
                ));
            }
        }
    }

    fn title(&self) -> String {
        "Live session".into()
    }

    fn title_color(&self) -> Color32 {
        hex_color!("#ffffff")
    }

    fn bg_color(&self) -> Color32 {
        hex_color!("#C7077A")
    }
}

fn lin_scale(x: f32, domain: (f32, f32), range: (f32, f32)) -> f32 {
    (x - domain.0) / (domain.1 - domain.0) * (range.1 - range.0) + range.0
}

fn exp_scale(x: f32, domain: (f32, f32), range: (f32, f32)) -> f32 {
    (x - domain.0) / (domain.1 - domain.0) * (range.1 - range.0) + range.0
}
