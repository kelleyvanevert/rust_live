use std::thread::sleep;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, FromSample, SampleRate, SizedSample, StreamConfig};

use crate::modulate::Modulation;
use crate::osc::*;

pub fn music() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("Failed to find a default output device");

    let config = StreamConfig {
        channels: 1,
        sample_rate: SampleRate(44_100),
        buffer_size: BufferSize::Default,
    };

    run::<f32>(&device, &config).unwrap();
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f64>,
{
    let mut o1 = Osc::default();
    o1.map("v".into(), "volume".into());
    o1.map("sq".into(), "squareness".into());
    o1.map("a".into(), "frequency".into());
    // o1.apply("frequency".into(), 220.0);

    let mut o2 = Osc::default();
    o2.map("v".into(), "volume".into());
    o2.map("sq".into(), "squareness".into());
    o2.map("b".into(), "frequency".into());
    // o2.apply("frequency".into(), 4.0 * 440.0);

    let mut o3 = Osc::default();
    o3.map("v".into(), "volume".into());
    o3.map("sq".into(), "squareness".into());
    o3.map("c".into(), "frequency".into());
    // o3.apply("frequency".into(), 4.0 * 440.0);

    let mut kick = Sample::new("../editor/res/samples/Kick 90s 1.wav").delay(1.0);
    kick.apply("repeat".into(), 0.0);

    let n = Mix::default()
        .add(Box::new(o1))
        .add(Box::new(o2))
        .add(Box::new(o3))
        .add(Box::new(kick));

    let mut w = Wrapper::new(Box::new(n));

    let frontend = w.get_frontend();

    let mut next_value = move || w.get_next_sample();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| write_data(data, 1, &mut next_value),
        err_fn,
        None,
    )?;

    let _ = frontend.send(("v".into(), 0.1));

    let bt = 0.3;

    let mut modulate_a = Modulation::new("a".into(), 220.0);
    let mut modulate_b = Modulation::new("b".into(), 4.0 * 440.0);
    let mut modulate_c = Modulation::new("c".into(), 4.0 * 440.0);
    let mut modulate_sq = Modulation::new("sq".into(), 0.3);

    modulate_a.schedule_transition(bt * 5.0, bt, 330.0);
    modulate_b.schedule_transition(bt * 5.0, bt, 3.0 * 440.0 * (3.0 / 5.0));
    modulate_c.schedule_transition(bt * 5.0, bt, 3.0 * 440.0 * (4.0 / 5.0));
    // modulate_sq.schedule_transition(bt * 5.0, bt, 1.0);

    modulate_a.schedule_transition(bt * 10.0, bt, 345.0 / 1.5);
    modulate_b.schedule_transition(bt * 10.0, bt, 3.0 * 440.0 * (3.2 / 3.0) * (2. / 3.));
    modulate_c.schedule_transition(bt * 10.0, bt, 3.0 * 440.0 * (3.2 / 3.0));
    // modulate_sq.schedule_transition(bt * 10.0, bt, 1.0);

    modulate_a.schedule_transition(bt * 15.0, bt, 440.0);
    modulate_b.schedule_transition(bt * 15.0, bt, 3.0 * 440.0 * (4.5 / 5.0));
    modulate_c.schedule_transition(bt * 15.0, bt, 3.0 * 440.0 * (3.0 / 5.0) * 1.75);
    // modulate_sq.schedule_transition(bt * 15.0, bt, 1.0);

    modulate_a.schedule_transition(bt * 20.0, bt, 220.0);
    modulate_b.schedule_transition(bt * 20.0, bt, 4.0 * 440.0);
    modulate_c.schedule_transition(bt * 20.0, bt, 4.0 * 440.0);
    modulate_sq.schedule_transition(bt * 20.0, bt, 0.8);

    modulate_a.schedule_transition(bt * 25.0, bt, 330.0);
    modulate_b.schedule_transition(bt * 25.0, bt, 3.0 * 440.0 * (3.0 / 5.0));
    modulate_c.schedule_transition(bt * 25.0, bt, 3.0 * 440.0 * (4.0 / 5.0));
    // modulate_sq.schedule_transition(bt * 20.0, bt, 0.8);

    modulate_a.schedule_transition(bt * 30.0, bt, 345.0 / 1.5);
    modulate_b.schedule_transition(bt * 30.0, bt, 3.0 * 440.0 * (3.2 / 3.0) * (2. / 3.));
    modulate_c.schedule_transition(bt * 30.0, bt, 3.0 * 440.0 * (3.2 / 3.0));
    // modulate_sq.schedule_transition(bt * 20.0, bt, 0.8);

    modulate_a.schedule_transition(bt * 35.0, bt, 440.0 * 0.5);
    modulate_b.schedule_transition(bt * 35.0, bt, 3.0 * 440.0 * (4.5 / 5.0) * 0.5);
    modulate_c.schedule_transition(bt * 35.0, bt, 3.0 * 440.0 * (3.0 / 5.0) * 1.75 * 0.5);
    // modulate_sq.schedule_transition(bt * 20.0, bt, 0.8);

    stream.play()?;

    let mut time = 0.0;

    loop {
        time += 0.001;
        let _ = frontend.send(modulate_a.get_message(time));
        let _ = frontend.send(modulate_b.get_message(time));
        let _ = frontend.send(modulate_c.get_message(time));
        let _ = frontend.send(modulate_sq.get_message(time));

        sleep(Duration::MILLISECOND);
    }

    // Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: SizedSample + FromSample<f64>,
{
    for frame in output.chunks_mut(channels) {
        let s = next_sample() as f64;
        let s = T::from_sample(s);

        for (_, sample) in frame.iter_mut().enumerate() {
            *sample = s;
        }
    }
}
