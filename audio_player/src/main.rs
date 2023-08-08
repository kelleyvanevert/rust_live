use std::time::Instant;

use crate::read_audio_file::read_audio_file;

mod read_audio_file;

fn main() {
    // Get the first command line argument.
    let args: Vec<String> = std::env::args().collect();
    let filepath = args.get(1).expect("file path not provided");
    println!("reading auto file: {}", filepath);

    let t0 = Instant::now();
    let info = read_audio_file(&filepath);
    println!("num samples: {}", info.samples.len());
    println!(
        "min: {:?}",
        info.samples.iter().fold(0.0f32, |a, &b| { a.min(b) })
    );
    println!(
        "max: {:?}",
        info.samples.iter().fold(0.0f32, |a, &b| { a.max(b) })
    );
    println!("len (s): {:?}", info.length_seconds);
    println!("channels: {:?}", info.num_channels);
    println!("took: {:?}", t0.elapsed());
}
