// trait SampleSink {

// }

#[macro_use]
mod node;

mod error;
mod interface;
mod pool;
mod recorder;
mod sink;

fn main() {
    let pool_node = pool::run_lru_pool().unwrap();
    let sink_node = sink::run_writer("record.wav", Some(pool_node.clone_sender())).unwrap();
    let recorder_node = recorder::run_recorder(sink_node.clone_sender()).unwrap();
    let interface = interface::run_shell_interface(pool_node.clone_sender()).unwrap();

    interface.join();

    recorder_node.stop();
    sink_node.stop();
    pool_node.stop();

    println!("exit");
}
