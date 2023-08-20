use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use egui::*;
use itertools::Itertools;
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencySpectrum};

use super::dash::{Dash, DASH_HEIGHT};

const MIN_FREQ: f32 = 20.0;
const MAX_FREQ: f32 = 20_000.0;
const FFT_SAMPLE_BUFFER_SIZE: usize = 4096 * 2;

struct RecordingInfo {
    quit: bool,
    sample_rate: u32,
    latest_samples: [f32; FFT_SAMPLE_BUFFER_SIZE],
    spectrum: Option<FrequencySpectrum>,
}

pub struct SessionDash {
    recording: Option<Arc<Mutex<RecordingInfo>>>,

    val_max: f32,
    mem_highest: Option<Vec<f32>>,
}

impl SessionDash {
    pub fn new() -> Self {
        Self {
            recording: None,
            val_max: 0.005,
            mem_highest: None,
        }
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
            latest_samples: [0.0; FFT_SAMPLE_BUFFER_SIZE],
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

                let n = input.len() / channels as usize;

                info.latest_samples.rotate_left(n);

                let i0 = info.latest_samples.len() - n;
                for (i, frame) in input.chunks(channels as _).enumerate() {
                    info.latest_samples[i0 + i] = frame[0].to_sample::<f32>();
                }

                // info.latest_samples = input
                //     .chunks(channels as _)
                //     .map(|frame| {
                //         //
                //         frame[0].to_sample::<f32>()
                //     })
                //     .collect::<Vec<_>>();

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
            }

            let info_2 = info.clone();

            let stream_config: StreamConfig = config.clone().into();
            // stream_config.buffer_size = BufferSize::Fixed(1024);

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
        specto_rect.min.y += 40.0;
        let xmin = specto_rect.min.x;
        // let xmax = specto_rect.max.x;
        // let ymin = specto_rect.min.y;
        let ymax = specto_rect.max.y;
        // let w = specto_rect.width();
        let h = specto_rect.height();

        // // debug
        // ui.painter()
        //     .rect_filled(specto_rect, 0.0, hex_color!("#cc000077"));

        let dm = 0.00005;
        // let dvm = 0.00005;

        let bin_width = 12.0;
        let num_bins = (specto_rect.width() / bin_width) as usize;
        let bin_width_frac = bin_width as f32 / specto_rect.width();

        let mut bins = vec![vec![]; num_bins + 1];

        if let Some(info) = &self.recording {
            let info = info.lock().unwrap();
            if let Some(spectrum) = &info.spectrum {
                // println!("max: {:?}, len: {:?}", valmax, spectrum.data().len());

                // for (fr, fr_val) in spectrum.data().iter() {
                //     println!("{}Hz => {}", fr, fr_val)
                // }

                let _line_points = spectrum
                    .data()
                    .iter()
                    .map(|&(freq, val)| {
                        // let y = lin_scale(val.val(), (0.0, valmax), (0.0, 1.0));
                        let x = exp_scale(
                            freq.val().log2(),
                            (MIN_FREQ.log2(), MAX_FREQ.log2()),
                            (0.0, 1.0),
                        );

                        let i = (x / bin_width_frac) as usize;
                        bins[i].push(val.val());

                        pos2(x, val.val())
                    })
                    .collect_vec();

                let bins = bins
                    .into_iter()
                    .map(|bin| {
                        if bin.len() == 0 {
                            0.0
                        } else {
                            bin.iter().sum::<f32>() / bin.len() as f32
                        }
                    })
                    .collect_vec();

                let spectrum_val_max = bins.iter().cloned().reduce(f32::max).unwrap_or(0.0);
                // println!("new max: {} cmp {}", spectrum_val_max, self.val_max);
                let val_max = spectrum_val_max.max(self.val_max /*- dvm*/);
                self.val_max = val_max;

                if let Some(highest) = &self.mem_highest {
                    let highest = resample_simple(highest, bins.len());
                    self.mem_highest = Some(
                        highest
                            .iter()
                            .zip(bins.iter())
                            .map(|(&highest, &curr)| (highest - dm).max(curr))
                            .collect_vec(),
                    );
                } else {
                    self.mem_highest = Some(bins.clone());
                }

                if let Some(highest) = &self.mem_highest {
                    // painter.add(Shape::line(
                    //     highest
                    //         .iter()
                    //         .enumerate()
                    //         .map(|(i, val)| {
                    //             let x = xmin + i as f32 * bin_width;
                    //             let y = ymax - (val / val_max) * h;
                    //             pos2(x, y)
                    //         })
                    //         .collect_vec(),
                    //     Stroke::new(1.0, hex_color!("#ffffff11")),
                    // ));

                    for (i, val) in highest.iter().enumerate() {
                        let x = xmin + i as f32 * bin_width;
                        let y = ymax - (val / val_max) * h;

                        painter.add(Shape::rect_filled(
                            Rect {
                                min: pos2(x, y),
                                max: pos2(x + bin_width - 2.0, ymax),
                            },
                            0.0,
                            hex_color!("#ffffff11"),
                        ));
                    }
                }

                for (i, val) in bins.iter().enumerate() {
                    let x = xmin + i as f32 * bin_width;
                    let y = ymax - (val / val_max) * h;

                    painter.add(Shape::rect_filled(
                        Rect {
                            min: pos2(x, y),
                            max: pos2(x + bin_width - 2.0, ymax),
                        },
                        0.0,
                        hex_color!("#ffffff"),
                    ));
                }
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

#[allow(unused)]
fn lin_scale(x: f32, domain: (f32, f32), range: (f32, f32)) -> f32 {
    (x - domain.0) / (domain.1 - domain.0) * (range.1 - range.0) + range.0
}

fn exp_scale(x: f32, domain: (f32, f32), range: (f32, f32)) -> f32 {
    (x - domain.0) / (domain.1 - domain.0) * (range.1 - range.0) + range.0
}

fn resample_simple(source: &Vec<f32>, dest_len: usize) -> Vec<f32> {
    if source.len() == dest_len {
        return source.clone();
    }

    let ratio = source.len() as f32 / dest_len as f32;

    let mut output = vec![0.0; dest_len + 1];

    for i in 0..source.len() {
        let a = i as f32 / ratio;
        let b = (i + 1) as f32 / ratio;
        if a as usize == b as usize {
            output[a as usize] += source[i] / ratio;
        } else {
            output[a as usize] += source[i] * (b.floor() - a);
            output[b as usize] += source[i] * (b - b.floor());
        }
    }

    // the output is padded by 1 so that the flooring + working with `b` is easier
    output.pop();

    output
}

#[test]
fn test_resample_simple() {
    use float_cmp::*;

    fn assert_vec_approx_eq(a: Vec<f32>, b: Vec<f32>) {
        assert_eq!(a.len(), b.len());
        for i in 0..a.len() {
            assert_approx_eq!(f32, a[i], b[i]);
        }
    }

    let res = resample_simple(&vec![1.0; 11], 3);
    assert_vec_approx_eq(res, vec![1.0; 3]);

    let res = resample_simple(
        &vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        3,
    );
    assert_vec_approx_eq(
        res,
        vec![
            5.0 / 3.6666666,
            18.333333333 / 3.6666666,
            31.66666666 / 3.6666666,
        ],
    );
}
