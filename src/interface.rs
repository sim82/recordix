use crate::error::Result;
use crate::node;
use crate::node::CommandNode;
use crate::pool;

use byteorder::{NativeEndian, ReadBytesExt};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;

implement_command! {}

struct RustylineInterface {
    command_receiver: Receiver<Command>,
    pool_command: Sender<pool::Command>,
}

impl RustylineInterface {
    fn command(&mut self, cmd: &str) -> Result<()> {
        match cmd {
            "avg_level" => {
                let (send, recv) = channel();

                self.pool_command
                    .send(pool::Command::ApplyToLast(
                        48000,
                        Box::new(move |buf| {
                            let num_samples = buf.len() / 2;
                            let mut cursor = std::io::Cursor::new(buf);
                            let mut avg = 0f64;
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            for _i in 0..num_samples {
                                let sample = cursor.read_i16::<NativeEndian>().unwrap();
                                avg += (sample as f64).abs() * (1f64 / num_samples as f64);
                            }
                            // let sum = buf.iter().fold(0f64, |acc, x| acc + *x as f64);
                            // println!("avg: {}", avg);
                            send.send(avg).expect("send error");
                        }),
                    ))
                    .unwrap();

                if let Ok(avg) = recv.recv() {
                    println!("avg: {}", avg);
                }
            }
            _ => println!("unknown command"),
        };
        Ok(())
    }

    fn mainloop(&mut self) -> Result<()> {
        loop {
            let mut rl = Editor::<()>::new();

            match rl.readline(">") {
                Ok(input) => self.command(&input)?,
                Err(ReadlineError::Eof) => {
                    println!("eof");
                    break;
                }
                Err(ReadlineError::Interrupted) => {
                    println!("interrupted");
                    break;
                }
                Err(x) => {
                    println!("readline error {:?}", x);
                    break;
                }
            };

            match self.command_receiver.try_recv() {
                Ok(Command::Node(node::Command::Stop)) => {
                    println!("rustyline interface stop");
                    break;
                }
                Err(_) => (),
            }
        }

        Ok(())
    }
}

pub fn run_shell_interface(pool_command: Sender<pool::Command>) -> Result<CommandNode<Command>> {
    let (send, recv) = channel();

    let mut interface = RustylineInterface {
        command_receiver: recv,
        pool_command: pool_command,
    };
    let handle = spawn(move || {
        interface.mainloop().expect("mainloop error");
    });
    Ok(CommandNode::new(handle, send))
}
