use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use crate::{Data, save_record_data};

pub fn create_client() {
    let args = std::env::args().collect::<Vec<String>>();
    // if args[1] is not none we take that as the server_name else localhost
    let default = "localhost".to_string();
    let server_name = format!("{}:3333", args.get(1).unwrap_or(&default).to_string());
    println!("Connecting to server: {}", server_name);
    match TcpStream::connect(server_name) {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333");

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
                let mut data = vec![0u8; len as usize];
                stream.read_exact(&mut data).unwrap();
                println!("Received data: {:?}", data);
                // convert back to data type
                let data: Vec<Data> = bincode::deserialize(&data).unwrap();
                println!("Received data: {:?}", data);
                for d in data {
                    save_record_data(d);
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}