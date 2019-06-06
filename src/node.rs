use crate::error;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

#[macro_export]
macro_rules! implement_command {
    ($($y:tt) *) => { // TODO: I'm not sure if this is exactly how this should be handled...
        pub enum Command {
            Node(node::Command),
            $($y)*
        }

        impl From<crate::node::Command> for Command {
            fn from(cmd: crate::node::Command) -> Self {
                Command::Node(cmd)
            }
        }
    };
}

pub enum Command {
    Stop,
}

pub struct CommandNode<T: Send> {
    command_sender: Sender<T>,
    join_handle: std::thread::JoinHandle<()>,
    // stop_command: T,
}

impl<T: Send + From<Command>> CommandNode<T> {
    pub fn new(join_handle: JoinHandle<()>, sender: Sender<T>) -> Self {
        CommandNode {
            join_handle: join_handle,
            command_sender: sender,
            // stop_command: stop_command,
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
            // stop_command,
        } = self;

        command_sender.send(Command::Stop.into()).unwrap();
        join_handle.join().unwrap();
    }

    pub fn join(self) {
        let CommandNode { join_handle, .. } = self;

        join_handle.join().unwrap();
    }
}

// type Factory = Box<Fn() -> Box<CommandNode<From<Command>>>>;

// struct Manager {
//     factories: std::collections::HashMap<String, Factory>,
// }

#[cfg(test)]
mod tests {
    use super::Command;
    use super::CommandNode;
    use std::sync::mpsc::channel;
    use std::thread::spawn;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    fn test_node_lifecycle() {
        let (send, recv) = channel();

        let handle = spawn(move || loop {
            match recv.recv() {
                Ok(Command::Stop) => break,
                _ => (),
            }
        });
        let node = CommandNode::new(handle, send);

        let sender2 = node.clone_sender();
        let handle2 = spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            sender2.send(Command::Stop).expect("join failed");
        });

        let t1 = Instant::now();
        node.join();
        assert!(
            Instant::now().duration_since(t1) > Duration::from_millis(300),
            "join returned too fast",
        );
        handle2.join().expect("join failed");
    }
}
