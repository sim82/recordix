use super::error::{Error, Result};
use byteorder::{NativeEndian, ReadBytesExt};
use hound;
use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;

pub enum Command {
    Append(Vec<u8>),
    ApplyToLast(usize, Box<Fn(Vec<u8>) + Send>),
    Stop,
}

type HoundWavWriter = hound::WavWriter<std::io::BufWriter<std::fs::File>>;

struct WaveWriter {
    command_receiver: Receiver<Command>,
    command_sender: Sender<Command>,
    last_buffers: VecDeque<Vec<u8>>,
    writer: HoundWavWriter,
}

impl WaveWriter {
    fn mainloop(&mut self) -> Result<()> {
        loop {
            match self.command_receiver.recv()? {
                Command::Stop => {
                    println!("sink stop");
                    break;
                }
                Command::Append(buf) => {
                    let num_samples = buf.len() / 4;
                    let mut cursor = Cursor::new(&buf);

                    for i in 0..num_samples {
                        let left = cursor.read_i16::<NativeEndian>()?;
                        let right = cursor.read_i16::<NativeEndian>()?;

                        // println!("{} {}", left, right);
                        self.writer.write_sample(left)?;
                        self.writer.write_sample(right)?;
                    }
                    self.writer.flush();

                    println!("wrote {} samples", num_samples);

                    self.last_buffers.push_back(buf);
                    const STORED_BUF_MAX_SIZE: usize = 1024 * 1024 * 10;
                    while self.stored_buffer_size() > STORED_BUF_MAX_SIZE {
                        self.last_buffers.pop_front();
                    }
                }
                Command::ApplyToLast(num_samples, f) => {
                    f(self
                        .last_buffers
                        .back()
                        .cloned()
                        .unwrap_or_else(|| vec![0u8; 0]));
                }
            }
        }
        Ok(())
    }

    fn stored_buffer_size(&self) -> usize {
        self.last_buffers.iter().fold(0, |acc, x| acc + x.len())
    }
}

pub fn run_writer<P: AsRef<std::path::Path>>(
    filename: P,
) -> Result<(std::thread::JoinHandle<()>, Sender<Command>)> {
    let (send, revc) = channel();

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WaveWriter {
        command_receiver: revc,
        command_sender: send,
        last_buffers: VecDeque::new(),
        writer: HoundWavWriter::create(filename, spec)?,
    };
    let sender = writer.command_sender.clone();
    let handle = spawn(move || {
        writer.mainloop();
    });
    Ok((handle, sender))
}

impl From<hound::Error> for Error {
    fn from(err: hound::Error) -> Error {
        Error::Data(format!("write_sample failed: {}", err))
    }
}
