#![feature(let_chains)]
#![feature(slice_group_by)]
#![feature(duration_constants)]

use music::music;

mod music;
mod osc;
mod read_audio_file;
mod util;

fn main() {
    music();
}
