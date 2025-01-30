use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

use request::Request;

mod request;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap();
                let Some(request) = Request::from_buffer(&buf[..n]) else {
                    break;
                };
                match request.target.as_str() {
                    "/" => stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap(),
                    _ => stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap()
                };

                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
