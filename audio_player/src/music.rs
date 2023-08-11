use rodio::{OutputStream, Source};
use std::{thread::sleep, time::Duration};

use crate::{generating::*, util::ease_cubic_in_out};

pub fn music() {
    let pattern = vec![
        vec![
            silent(),
            Params::default().freq(220.0).vol(0.5),
            (4.0 * 440.0).into(),
            (4.0 * 440.0).into(),
        ],
        vec![
            silent(),
            330.0.into(),
            (3.0 * 440.0 * (3.0 / 5.0)).into(),
            (3.0 * 440.0 * (4.0 / 5.0)).into(),
        ],
        vec![
            silent(),
            (345.0 / 1.5).into(),
            (3.0 * 440.0 * (3.2 / 3.0) * (2. / 3.)).into(),
            (3.0 * 440.0 * (3.2 / 3.0)).into(),
        ],
        vec![
            silent(),
            440.0.into(),
            (3.0 * 440.0 * (4.5 / 5.0)).into(),
            (3.0 * 440.0 * (3.0 / 5.0) * 1.75).into(),
        ],
        vec![
            Params::default().freq(110.0).sq(0.8).vol(0.5),
            Params::default().freq(220.0).sq(0.8).vol(0.4),
            Params::default().freq(4.0 * 440.0).vol(0.4),
            Params::default().freq(4.0 * 440.0).vol(0.4),
        ],
        vec![
            Params::default().freq(110.0).sq(0.8).vol(0.5),
            Params::default().freq(330.0).sq(0.5),
            Params::default().freq(3.0 * 440.0 * (3.0 / 5.0)).sq(0.5),
            Params::default().freq(3.0 * 440.0 * (4.0 / 5.0)).sq(0.5),
        ],
        vec![
            Params::default().freq(130.0).sq(0.8).vol(0.5),
            Params::default().freq(345.0 / 1.5).sq(0.8),
            Params::default()
                .freq(3.0 * 440.0 * (3.2 / 3.0) * (2. / 3.))
                .sq(0.8),
            Params::default().freq(3.0 * 440.0 * (3.2 / 3.0)).sq(0.8),
        ],
        vec![
            Params::default().freq(175.0).sq(0.8).vol(0.5),
            Params::default().freq(440.0 * 0.5),
            Params::default().freq(3.0 * 440.0 * (4.5 / 5.0) * 0.5),
            Params::default().freq(3.0 * 440.0 * (3.0 / 5.0) * 1.75 * 0.5),
        ],
    ];

    let oscs: Vec<_> = pattern[0].iter().map(|&p| Osc::sine(p)).collect();
    let frontends: Vec<_> = oscs.iter().map(|o| o.frontend()).collect();

    let osc = Mix::new(oscs);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let _res = stream_handle.play_raw(osc.convert_samples());

    let bt = 500;

    sleep(Duration::from_millis(bt * 5));

    for i in 0..pattern.len() - 1 {
        let a = &pattern[i];
        let b = &pattern[i + 1];

        for t in 0..bt {
            for j in 0..frontends.len() {
                let _ = frontends[j].send(lerp_params(
                    ease_cubic_in_out(t as f32 / bt as f32),
                    a[j],
                    b[j],
                ));
            }
            sleep(Duration::MILLISECOND);
        }

        sleep(Duration::from_millis(bt * 4));
    }

    sleep(Duration::from_millis(bt * 1));
}

/*

let sound = |a = 440 hz, b = 440 hz| sin(f: a) + sin(f: b)

let p = |n = A3| {
    [n; 6] + [n -= 7t; 5] +
}

let p = (A3..... C#2..... D2..... B#2.....)

play sound(a: p, b:)

# play A

@{
    120bpm;
    5bx { }
}

*/
