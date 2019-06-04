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
        let CommandNode { join_handle, .. } = self;

        join_handle.join().unwrap();
    }
}

mod tests {
    use super::CommandNode;
    use std::sync::mpsc::channel;
    use std::thread::spawn;
    use std::time::Duration;
    use std::time::Instant;
    enum Command {
        Stop,
    }
    #[test]
    fn test_node_lifecycle() {
        let (send, recv) = channel();

        let handle = spawn(move || loop {
            match recv.recv() {
                Ok(Command::Stop) => break,
                _ => (),
            }
        });
        let node = CommandNode::new(handle, send, Command::Stop);

        let sender2 = node.clone_sender();
        let handle2 = spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            sender2.send(Command::Stop).expect("join failed");
        });

        let t1 = Instant::now();
        println!("waiting for node to stop ...");
        node.join();
        println!("stopped");
        assert!(Instant::now().duration_since(t1) > Duration::from_millis(300));
        handle2.join().expect("join failed");
    }
}
