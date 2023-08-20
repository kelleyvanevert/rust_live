use itertools::Itertools;
use std::io::ErrorKind;
use symphonia::core::{
    audio::SampleBuffer,
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    errors::Error,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

pub struct AudioTrackInfo {
    pub samples: Vec<f32>,
    pub num_channels: usize,
    pub length_seconds: f64,
}

impl AudioTrackInfo {
    pub fn get_mono_samples(&self) -> Vec<f32> {
        let n = self.num_channels;
        self.samples
            .iter()
            .enumerate()
            .filter(|(i, _)| i % n == 0)
            .map(|(_, &s)| s)
            .collect_vec()
    }
}

// This is just copied pretty much directly from https://github.com/pdeljanov/Symphonia/blob/master/GETTING_STARTED.md
pub fn read_audio_file(filepath: &str) -> AudioTrackInfo {
    // Open the media source.
    let src = std::fs::File::open(filepath).expect("failed to open media");

    // Create the media source stream.
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    // Use the default options for metadata and format readers.
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    // Probe the media source.
    let probed = symphonia::default::get_probe()
        .format(&Hint::new(), mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    // Get the instantiated format reader.
    let mut format = probed.format;

    // Find the first audio track with a known (decodeable) codec.
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .expect("no supported audio tracks");

    // Use the default options for the decoder.
    let dec_opts: DecoderOptions = Default::default();

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");

    // Store the track identifier, it will be used to filter packets.
    let track_id = track.id;

    let tb = track.codec_params.time_base;
    let nf = track.codec_params.n_frames;

    let num_channels = track
        .codec_params
        .channels
        .map(|chs| chs.count())
        .unwrap_or(1);

    let mut samples: Vec<f32> = vec![];

    // The decode loop.
    loop {
        // Get the next packet from the media format.
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::ResetRequired) => {
                // The track list has been changed. Re-examine it and create a new set of decoders,
                // then restart the decode loop. This is an advanced feature and it is not
                // unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
                // for chained OGG physical streams.
                unimplemented!();
            }
            Err(Error::IoError(err))
                if err.kind() == ErrorKind::UnexpectedEof
                    && err.to_string() == "end of stream"
                    && samples.len() > 0 =>
            {
                // we're done :)
                break;
            }
            Err(err) => {
                // A unrecoverable error occured, halt decoding.
                panic!("{}", err);
            }
        };

        // Consume any new metadata that has been read since the last packet.
        while !format.metadata().is_latest() {
            // Pop the old head of the metadata queue.
            format.metadata().pop();

            // Consume the new metadata at the head of the metadata queue.
        }

        // If the packet does not belong to the selected track, skip over it.
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Create a sample buffer that matches the parameters of the decoded audio buffer.
                let mut sample_buf =
                    SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());

                // Copy the contents of the decoded audio buffer into the sample buffer whilst performing
                // any required conversions.
                sample_buf.copy_interleaved_ref(decoded);

                // The interleaved f32 samples can be accessed as follows.
                let new_samples = sample_buf.samples();
                samples.extend_from_slice(new_samples);
            }
            Err(Error::IoError(_)) => {
                // The packet failed to decode due to an IO error, skip the packet.
                continue;
            }
            Err(Error::DecodeError(_)) => {
                // The packet failed to decode due to invalid data, skip the packet.
                continue;
            }
            Err(err) => {
                // An unrecoverable error occured, halt decoding.
                panic!("{}", err);
            }
        }
    }

    let time = tb
        .unwrap()
        .calc_time(nf.unwrap_or_else(|| (samples.len() / num_channels) as u64));

    let length_seconds = time.seconds as f64 + time.frac;

    AudioTrackInfo {
        samples,
        num_channels,
        length_seconds,
    }
}
