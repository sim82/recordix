use super::error::Result;
use super::CommandNode;
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;

#[allow(unused)]
pub enum Command {
    Append(Vec<u8>),
    ApplyToLast(usize, Box<Fn(Vec<u8>) + Send>),
    Stop,
}

struct LruPool {
    command_receiver: Receiver<Command>,
    last_buffers: VecDeque<Vec<u8>>,
}

impl LruPool {
    fn mainloop(&mut self) -> Result<()> {
        loop {
            match self.command_receiver.recv()? {
                Command::Stop => {
                    println!("pool stop");
                    break;
                }
                Command::Append(buf) => {
                    self.last_buffers.push_back(buf);
                    const STORED_BUF_MAX_SIZE: usize = 1024 * 1024 * 10;
                    while self.stored_buffer_size() > STORED_BUF_MAX_SIZE {
                        self.last_buffers.pop_front();
                    }
                }
                Command::ApplyToLast(_num_samples, f) => {
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

pub fn run_lru_pool() -> Result<CommandNode<Command>> {
    let (send, revc) = channel();

    let mut pool = LruPool {
        command_receiver: revc,
        last_buffers: VecDeque::new(),
    };
    let handle = spawn(move || {
        pool.mainloop().expect("mainloop error");
    });
    Ok(CommandNode::new(handle, send, Command::Stop))
}
