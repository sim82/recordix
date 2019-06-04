use crate::error;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

pub struct CommandNode<T: Send> {
    command_sender: Sender<T>,
    join_handle: std::thread::JoinHandle<()>,
    stop_command: T,
}

impl<T: Send> CommandNode<T> {
    pub fn new(join_handle: JoinHandle<()>, sender: Sender<T>, stop_command: T) -> Self {
        CommandNode {
            join_handle: join_handle,
            command_sender: sender,
            stop_command: stop_command,
        }
    }
    #[allow(unused)]
    pub fn send(&self, command: T) -> error::Result<()> {
        self.command_sender.send(command)?;
        Ok(())
    }

    pub fn clone_sender(&self) -> Sender<T> {
        self.command_sender.clone()
    }
    pub fn stop(self) {
        // FIXME: is there a more elegant way to consume self? the compiler complains about borrowing after partial move etc.
        let CommandNode {
            join_handle,
            command_sender,
            stop_command,
        } = self;

        command_sender.send(stop_command).unwrap();
        join_handle.join().unwrap();
    }

    pub fn join(self) {
        let CommandNode {
            join_handle,
            ..
            // command_sender,
            // stop_command,
        } = self;

        join_handle.join().unwrap();
    }
}
