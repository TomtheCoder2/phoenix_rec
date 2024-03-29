use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::Duration;
use bincode::deserialize;
use crate::Data;

static DATA_QUEUE: Mutex<Vec<Data>> = Mutex::new(Vec::new());

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    loop {
        stream.set_read_timeout(Option::from(Duration::from_micros(10))).unwrap();
        match stream.read(&mut data) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                println!("loop");
                // print received data
                let text = String::from_utf8_lossy(&data).to_string().trim_matches(char::from(0)).to_string();
                println!("Received data: {}, len: {}", text, text.len());
                if text == "hello" {
                    stream.write(b"hello").unwrap();
                }

                if text.contains("close") {
                    println!("Terminating connection");
                    stream.shutdown(Shutdown::Both).unwrap();
                    break;
                }
            }
            Err(_e) => {
                if DATA_QUEUE.lock().unwrap().is_empty() {
                    continue;
                }
                // send all data from the queue
                let mut queue = DATA_QUEUE.lock().unwrap().clone();
                println!("Queue: {:?}", queue);
                let data = bincode::serialize(&queue);
                let data_type: Vec<Data> = deserialize(&bincode::serialize(&queue).unwrap()).unwrap();
                println!("Data type: {:?}", data_type);
                if queue.len() > 0 {
                    println!("Sending data: {:?}", queue);
                    match data {
                        Ok(d) => {
                            // first send the length of the data
                            stream.write(&(d.len() as u32).to_le_bytes()).unwrap();
                            dbg!(&(d.len() as u32).to_le_bytes());
                            stream.write(&d).unwrap();
                            println!("Sent data: {:?}", d);
                            queue.clear();
                            DATA_QUEUE.lock().unwrap().clear();
                        }
                        Err(e) => {
                            println!("Failed to serialize data: {}", e);
                        }
                    }
                }
            }
        };
    }
}

pub fn create_server() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
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
    println!("Terminated_server");
    // close the socket server
    drop(listener);
}

pub fn add_data(data: Data) {
    println!("Adding data: {:?}", data);
    DATA_QUEUE.lock().unwrap().push(data);
}
