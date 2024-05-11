use crate::client::PORT;
use crate::{Data, SERVER};
use bincode::deserialize;
use lz4_compression::prelude::compress;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static DATA_QUEUE: Mutex<Vec<Data>> = Mutex::new(Vec::new());
static STOP_SERVER: AtomicBool = AtomicBool::new(false);

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    loop {
        stream
            .set_read_timeout(Option::from(Duration::from_micros(10)))
            .unwrap();
        match stream.read(&mut data) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                debug!("loop");
                // print received data
                let text = String::from_utf8_lossy(&data)
                    .to_string()
                    .trim_matches(char::from(0))
                    .to_string();
                debug!("Received data: {}, len: {}", text, text.len());
                if text == "hello" {
                    stream.write(b"hello").unwrap();
                }

                if text.contains("close") {
                    println!("Terminating connection");
                    stream.shutdown(Shutdown::Both).unwrap();
                    SERVER.store(false, std::sync::atomic::Ordering::SeqCst);
                    break;
                }
            }
            Err(_e) => {
                if STOP_SERVER.load(std::sync::atomic::Ordering::SeqCst) {
                    println!("Terminating connection");
                    stream.write(&(0u32).to_le_bytes()).unwrap();
                    stream.shutdown(Shutdown::Both).unwrap();
                    SERVER.store(false, std::sync::atomic::Ordering::SeqCst);
                    break;
                }
                if DATA_QUEUE.lock().unwrap().is_empty() {
                    continue;
                }
                // send all data from the queue
                let mut queue = DATA_QUEUE.lock().unwrap().clone();
                debug!("Queue: {:?}", queue);
                let data = bincode::serialize(&queue);
                let data_type: Vec<Data> =
                    deserialize(&bincode::serialize(&queue).unwrap()).unwrap();
                debug!("Data type: {:?}", data_type);
                debug!("Sending data: {:?}", queue);
                match data {
                    Ok(d) => {
                        debug!("length before compression: {}", d.len());
                        let d = compress(&d);
                        debug!("length after compression: {}", d.len());
                        // first send the length of the data
                        stream.write(&(d.len() as u32).to_le_bytes()).unwrap();
                        debug!("len: {:?}", (d.len() as u32).to_le_bytes());
                        stream.write(&d).unwrap();
                        debug!("Sent data: {:?}", d);
                        queue.clear();
                        DATA_QUEUE.lock().unwrap().clear();
                    }
                    Err(e) => {
                        debug!("Failed to serialize data: {}", e);
                    }
                }
            }
        };
    }
}

pub fn create_server() {
    if SERVER.load(std::sync::atomic::Ordering::SeqCst) {
        println!("Server already running");
        return;
    }
    SERVER.store(true, std::sync::atomic::Ordering::SeqCst);
    STOP_SERVER.store(false, std::sync::atomic::Ordering::SeqCst);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        println!("Incoming connection");
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream)
                });
                println!("Connection handled");
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    println!("Can't accept any more connections");
    // close the socket server
    drop(listener);
}

pub fn add_data(data: Data) {
    debug!("Adding data: {:?}", data);
    DATA_QUEUE.lock().unwrap().push(data);
}

pub fn stop_server() {
    SERVER.store(false, std::sync::atomic::Ordering::SeqCst);
    STOP_SERVER.store(true, std::sync::atomic::Ordering::SeqCst);
}
