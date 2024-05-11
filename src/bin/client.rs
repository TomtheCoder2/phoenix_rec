use phoenix_rec::client::create_client;
use phoenix_rec::debug;

fn main() {
    debug!("Debugging client.rs");
    let args = std::env::args().collect::<Vec<String>>();
    // if args[1] is not none we take that as the server_name else localhost
    let default = "localhost".to_string();
    let (_main_sender, thread_receiver) = std::sync::mpsc::channel();
    let (thread_sender, _main_receiver) = std::sync::mpsc::channel();
    create_client(
        args.get(1).unwrap_or(&default).to_string(),
        thread_receiver,
        thread_sender,
    );
    loop {}
}
