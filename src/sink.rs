use super::error::{Error, Result};
use super::pool;
use super::CommandNode;
use byteorder::{NativeEndian, ReadBytesExt};
use hound;
use std::io::Cursor;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;

pub enum Command {
    Append(Vec<u8>),
    Stop,
}

type HoundWavWriter = hound::WavWriter<std::io::BufWriter<std::fs::File>>;

struct WaveWriter {
    command_receiver: Receiver<Command>,
    command_sender: Sender<Command>,
    writer: HoundWavWriter,
    pool_sender: Option<Sender<pool::Command>>,
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

                    for _i in 0..num_samples {
                        let left = cursor.read_i16::<NativeEndian>()?;
                        let right = cursor.read_i16::<NativeEndian>()?;

                        // println!("{} {}", left, right);
                        self.writer.write_sample(left)?;
                        self.writer.write_sample(right)?;
                    }
                    self.writer.flush()?;

                    println!("wrote {} samples", num_samples);

                    if let Some(pool) = &self.pool_sender {
                        pool.send(pool::Command::Append(buf))?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn run_writer<P: AsRef<std::path::Path>>(
    filename: P,
    pool: Option<Sender<pool::Command>>,
) -> Result<CommandNode<Command>> {
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
        writer: HoundWavWriter::create(filename, spec)?,
        pool_sender: pool,
    };
    let sender = writer.command_sender.clone();
    let handle = spawn(move || {
        writer.mainloop().expect("mainloop error");
    });
    Ok(CommandNode::new(handle, sender, Command::Stop))
}

impl From<hound::Error> for Error {
    fn from(err: hound::Error) -> Error {
        Error::Data(format!("write_sample failed: {}", err))
    }
}
