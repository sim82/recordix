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
    stop_revc.recv().unwrap();
    recorder_send.send(recorder::Command::Stop);
    recorder_join.join();
    sink_send.send(sink::Command::Stop);
    sink_join.join();

    println!("exit");
}
