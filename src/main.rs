use libpulse_binding as pulse;
use libpulse_simple_binding as psimple;

use byteorder::{NativeEndian, ReadBytesExt};
use hound;
use psimple::Simple;
use pulse::stream::Direction;
use std::io::Cursor;

fn main() {
    let spec = pulse::sample::Spec {
        format: pulse::sample::SAMPLE_S16NE,
        channels: 2,
        rate: 48000,
    };
    assert!(spec.is_valid());

    let s = Simple::new(
        None,              // Use the default server
        "recordix",        // Our application’s name
        Direction::Record, // We want a playback stream
        None,              // Use the default device
        "input",           // Description of our stream
        &spec,             // Our sample format
        None,              // Use default channel map
        None,              // Use default buffering attributes
    )
    .unwrap();

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create("record.wav", spec).unwrap();

    const num_samples: usize = 48000;
    for _ in 0..10 {
        println!("recoding...");
        let mut buf = vec![0u8; num_samples * 2 * 2];
        s.read(&mut buf);

        println!("done");

        let mut cursor = Cursor::new(&buf);

        for i in 0..num_samples {
            let left = cursor.read_i16::<NativeEndian>().unwrap();
            let right = cursor.read_i16::<NativeEndian>().unwrap();

            // println!("{} {}", left, right);
            writer.write_sample(left);
            writer.write_sample(right);
        }
        writer.flush();
    }
}
