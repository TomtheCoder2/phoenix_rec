use crate::{save_record_data, Data};
use lz4_compression::prelude::decompress;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};

pub const PORT: u16 = 3333;
static CLIENT: AtomicBool = AtomicBool::new(false);

pub fn client_alive() -> bool {
    CLIENT.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn create_client(
    server_name: String,
    thread_receiver: Receiver<String>,
    thread_sender: Sender<String>,
) {
    if CLIENT.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    CLIENT.store(true, std::sync::atomic::Ordering::SeqCst);
    let server_name = format!("{}:{}", server_name, PORT);
    println!("Connecting to server: {}", server_name);
    match TcpStream::connect(server_name.clone()) {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333");
            thread_sender
                .send(format!("Successfully connected to server {}", server_name))
                .expect("Couldn't send to main thread");

            let msg = b"hello";

            stream.write(msg).unwrap();
            println!("Sent Hello, awaiting reply...");

            let mut data = [0u8; 5];
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == msg {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        println!("Unexpected reply: {}", text);
                    }
                }
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
            let mut data = [0u8; 4];
            // stream.write(b"close").unwrap();
            while let Ok(_) = stream.read(&mut data) {
                // first 4 bytes are the length of the data
                let len = u32::from_le_bytes(data);
                if len == 0 {
                    break;
                }
                let mut data = vec![0u8; len as usize];
                stream.read_exact(&mut data).unwrap();
                // debug!("Received data: {:?}", data);
                // convert back to data type
                let data = decompress(&data).unwrap();
                let data: Vec<Data> = bincode::deserialize(&data).unwrap();
                // debug!("Received data: {:?}", data);
                for d in data {
                    save_record_data(d);
                }
                // check if something has been sent over the main thread
                if let Ok(msg) = thread_receiver.try_recv() {
                    if msg == "exit" {
                        stream.write(b"close").unwrap();
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
    thread_sender
        .send("Terminated".to_string())
        .expect("Couldn't send to main thread");
    CLIENT.store(false, std::sync::atomic::Ordering::SeqCst);
}
