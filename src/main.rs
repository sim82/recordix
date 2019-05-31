use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
// trait SampleSink {

// }

mod error;
mod interface;
mod pool;
mod recorder;
mod sink;

pub struct CommandNode<T: Send> {
    command_sender: Sender<T>,
    join_handle: std::thread::JoinHandle<()>,
    stop_command: T,
}

impl<T: Send> CommandNode<T> {
    fn new(join_handle: JoinHandle<()>, sender: Sender<T>, stop_command: T) -> Self {
        CommandNode {
            join_handle: join_handle,
            command_sender: sender,
            stop_command: stop_command,
        }
    }
    #[allow(unused)]
    fn send(&self, command: T) -> error::Result<()> {
        self.command_sender.send(command)?;
        Ok(())
    }

    fn clone_sender(&self) -> Sender<T> {
        self.command_sender.clone()
    }
    fn stop(self) {
        // FIXME: is there a more elegant way to consume self? the compiler complains about borrowing after partial move etc.
        let CommandNode {
            join_handle,
            command_sender,
            stop_command,
        } = self;

        command_sender.send(stop_command).unwrap();
        join_handle.join().unwrap();
    }

    fn join(self) {
        let CommandNode {
            join_handle,
            ..
            // command_sender,
            // stop_command,
        } = self;

        join_handle.join().unwrap();
    }
}

fn main() {
    let pool_node = pool::run_lru_pool().unwrap();
    let sink_node = sink::run_writer("record.wav", Some(pool_node.clone_sender())).unwrap();
    let recorder_node = recorder::run_recorder(sink_node.clone_sender()).unwrap();
    let interface = interface::run_shell_interface().unwrap();

    // println!("waiting ...");
    // loop {
    //     std::thread::sleep_ms(1000);
    //     pool_node
    //         .send(pool::Command::ApplyToLast(
    //             48000,
    //             Box::new(|buf| {
    //                 let num_samples = buf.len() / 2;
    //                 let mut cursor = std::io::Cursor::new(buf);
    //                 let mut avg = 0f64;
    //                 std::thread::sleep_ms(500);
    //                 for i in 0..num_samples {
    //                     let sample = cursor.read_i16::<NativeEndian>().unwrap();
    //                     avg += (sample as f64).abs() * (1f64 / num_samples as f64);
    //                 }
    //                 // let sum = buf.iter().fold(0f64, |acc, x| acc + *x as f64);
    //                 println!("avg: {}", avg);
    //             }),
    //         ))
    //         .unwrap();

    //     if let Ok(_) = stop_revc.try_recv() {
    //         break;
    //     }
    // }

    interface.join();

    recorder_node.stop();
    sink_node.stop();
    pool_node.stop();

    println!("exit");
}
