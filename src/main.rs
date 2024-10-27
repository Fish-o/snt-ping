use network::start_network_loop;
use utils::Task;
mod network;
mod synchronize;
pub mod utils;

fn main() -> Result<(), std::io::Error> {
    println!("[MAIN] Hello! Cool to see you helping out for this rightous cause!");
    println!("[MAIN] Starting up...");
    let mut task = Task::blank();

    println!("[MAIN] Starting the sync thread...");
    let t = task.start_synchronizing();

    println!("[MAIN] Creating the network loop...");
    start_network_loop(&mut task);

    // This should theoretically be unreachable
    println!("[MAIN] Waiting for sync thead to join.");
    t.join().expect("Joining sync_thread failed");
    println!("[MAIN] Exiting");
    Ok(())
}
