use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

use request::{Request, Verb};
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
                match (request.verb, request.target.as_str()) {
                    (Verb::GET, "/") => stream
                        .write_all(Response::new(Status::OK).as_string().as_bytes())
                        .unwrap(),
                    (Verb::GET, target) if target.starts_with("/echo/") => {
                        let echo_back = request.target.strip_prefix("/echo/").unwrap();
                        let mut response = Response::new(Status::OK);
                        response.set_body(response::ContentType::Text, echo_back.to_owned());
                        stream.write_all(response.as_string().as_bytes()).unwrap();
                    }
                    (Verb::GET, "/user-agent") => {
                        let Some(user_agent) = request.headers.get("User-Agent") else {
                            stream
                                .write_all(Response::new(Status::NotFound).as_string().as_bytes())
                                .unwrap();
                            return;
                        };
                        let mut response = Response::new(Status::OK);
                        response.set_body(response::ContentType::Text, user_agent.to_owned());
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
