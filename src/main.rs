use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

use request::Request;
use response::{Response, Status};

mod request;
mod response;

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
                    "/" => stream
                        .write_all(Response::new(Status::OK).as_string().as_bytes())
                        .unwrap(),
                    target if target.starts_with("/echo/") => {
                        let echo_back = request.target.strip_prefix("/echo/").unwrap();
                        let mut response = Response::new(Status::OK);
                        response.set_body(response::ContentType::Text, echo_back.to_owned());

                        stream.write_all(response.as_string().as_bytes()).unwrap();
                    }
                    _ => stream
                        .write_all(Response::new(Status::NotFound).as_string().as_bytes())
                        .unwrap(),
                };

                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
