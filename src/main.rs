use libpulse_binding as pulse;
use libpulse_simple_binding as psimple;

use byteorder::{NativeEndian, ReadBytesExt};
use ctrlc;
use hound;
use psimple::Simple;
use pulse::stream::Direction;
use std::io::Cursor;
use std::sync::{Condvar, Mutex};

// trait SampleSink {

// }

mod error;
mod recorder;
mod sink;

fn main() {
    let (sink_join, sink_send) = sink::run_writer("record.wav").unwrap();
    let (recorder_join, recorder_send) = recorder::run_recorder(sink_send.clone()).unwrap();

    let (stop_send, stop_revc) = std::sync::mpsc::channel();

    ctrlc::set_handler(move || {
        println!("terminating ...");
        stop_send.send(()).unwrap();
    })
    .unwrap();

    println!("waiting ...");
    loop {
        std::thread::sleep_ms(1000);
        sink_send
            .send(sink::Command::ApplyToLast(
                48000,
                Box::new(|buf| {
                    let num_samples = buf.len() / 2;
                    let mut cursor = std::io::Cursor::new(buf);
                    let mut avg = 0f64;
                    for i in 0..num_samples {
                        let sample = cursor.read_i16::<NativeEndian>().unwrap();
                        avg += sample.abs() as f64 * (1f64 / num_samples as f64);
                    }
                    // let sum = buf.iter().fold(0f64, |acc, x| acc + *x as f64);
                    println!("avg: {}", avg);
                }),
            ))
            .unwrap();

        if let Ok(_) = stop_revc.try_recv() {
            break;
        }
    }

    recorder_send.send(recorder::Command::Stop);
    recorder_join.join();
    sink_send.send(sink::Command::Stop);
    sink_join.join();

    println!("exit");
}
