use std::{thread::sleep, time::Duration};

use rodio::{OutputStream, Source};

use crate::{
    generating::{Event, Osc},
    util::{ease_cubic_in_out, lerp},
};

pub fn music() {
    let a = 4.0 * 440.0;
    let b = 2.0 * 440.0 * (3.0 / 5.0);
    let c = 3.0 * 440.0 * (3.2 / 3.0);
    let d = 3.0 * 440.0 * (2.0 / 3.0);

    for i in 0..10 {
        println!(
            "{}, {}, {}",
            i,
            ease_cubic_in_out(i as f32 / 10.0),
            lerp(ease_cubic_in_out(i as f32 / 10.0), (0.0, 1.0), (a, b))
        );
    }

    let osc = Osc::sine(a, 0.5);

    let frontend = osc.frontend();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let _res = stream_handle.play_raw(osc.convert_samples());

    // let frontend = osc.frontend();

    sleep(Duration::from_millis(1500));

    for i in 0..300 {
        let f = lerp(ease_cubic_in_out(i as f32 / 300.0), (0.0, 1.0), (a, b));
        let _ = frontend.send(Event::SetFrequency(f));
        sleep(Duration::MILLISECOND);
    }

    sleep(Duration::from_millis(1500));

    for i in 0..300 {
        let f = lerp(ease_cubic_in_out(i as f32 / 300.0), (0.0, 1.0), (b, c));
        let _ = frontend.send(Event::SetFrequency(f));
        sleep(Duration::MILLISECOND);
    }

    sleep(Duration::from_millis(1500));

    for i in 0..300 {
        let f = lerp(ease_cubic_in_out(i as f32 / 300.0), (0.0, 1.0), (c, d));
        let _ = frontend.send(Event::SetFrequency(f));
        sleep(Duration::MILLISECOND);
    }

    sleep(Duration::from_millis(1500));
}
