
use super::error::{Error, Result};
use super::CommandNode;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;
pub enum Command {
    Stop,
}

struct RustylineInterface {
    command_receiver: Receiver<Command>,
}

impl RustylineInterface {
    fn mainloop(&mut self) -> Result<()> {
        loop {
            let mut rl = Editor::<()>::new();

            match rl.readline(">") {
                Ok(input) => println!("input: {}", input),
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
                Ok(Command::Stop) => {
                    println!("rustyline interface stop");
                    break;
                }
                Err(_) => ()
            }
        }

        Ok(())
    }
}


pub fn run_shell_interface() -> Result<CommandNode<Command>> {
    let (send, recv) = channel();

    let mut interface = RustylineInterface {
        command_receiver: recv,
    };
    let handle = spawn(move || {
        interface.mainloop();
    });
    Ok(CommandNode::new(handle, send, Command::Stop))

}