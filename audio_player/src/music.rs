use std::{thread::sleep, time::Duration};

use rodio::{OutputStream, Source};

use crate::generating::Osc;

pub fn music() {
    let osc = Osc::sine(440.0, 0.5);

    let frontend = osc.frontend();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let _res = stream_handle.play_raw(osc.convert_samples());

    // let frontend = osc.frontend();

    sleep(Duration::from_secs(10));
}
