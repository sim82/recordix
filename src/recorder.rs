use super::error::{Error, Result};
use super::sink;
use super::CommandNode;
use libpulse_binding as pulse;
use libpulse_simple_binding as psimple;
use psimple::Simple;
use pulse::stream::Direction;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum Command {
    Stop,
}

struct PulseAudioRecorder {
    command_receiver: Receiver<Command>,
    sink_command_sender: Sender<sink::Command>,
    pulse: Simple,
}

pub fn run_recorder(sink_command_sender: Sender<sink::Command>) -> Result<CommandNode<Command>> {
    let spec = pulse::sample::Spec {
        format: pulse::sample::SAMPLE_S16NE,
        channels: 2,
        rate: 48000,
    };
    assert!(spec.is_valid());

    let (send, recv) = channel();

    let mut recorder = PulseAudioRecorder {
        command_receiver: recv,
        sink_command_sender: sink_command_sender,
        pulse: Simple::new(
            None,              // Use the default server
            "recordix",        // Our application’s name
            Direction::Record, // We want a playback stream
            None,              // Use the default device
            "input",           // Description of our stream
            &spec,             // Our sample format
            None,              // Use default channel map
            None,              // Use default buffering attributes
        )?,
    };

    let latency = recorder.pulse.get_latency().unwrap();
    println!("latency: {}", latency);


    let join_handle = std::thread::spawn(move || {
        recorder.run().unwrap();
    });

    Ok(CommandNode::new(join_handle, send, Command::Stop))
}

impl PulseAudioRecorder {
    fn run(&mut self) -> Result<()> {
        loop {
            match self.command_receiver.try_recv() {
                Ok(Command::Stop) => {
                    println!("recorder stop");
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => (),
                Err(err) => return Err(err.into()),
            }

            const NUM_SAMPLES: usize = 48000;

            let mut buf = vec![0u8; NUM_SAMPLES * 2 * 2];
            self.pulse.read(&mut buf)?;

            self.sink_command_sender.send(sink::Command::Append(buf))?;
        }

        Ok(())
    }
}

impl From<pulse::error::PAErr> for Error {
    fn from(err: pulse::error::PAErr) -> Error {
        Error::Audio(format!("pulseaudio error: {}", err))
    }
}
