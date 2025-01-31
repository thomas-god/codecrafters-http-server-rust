use std::{collections::HashMap, env, fs, path::PathBuf};

use request::{Request, Verb};
use response::{Content, Response, Status};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

mod request;
mod response;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    let args = parse_args(env::args().collect());

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let file_folder = args.get("directory").map(|s| s.to_owned());
        tokio::spawn(async move {
            process_stream(stream, file_folder).await;
        });
    }
}

async fn process_stream(mut stream: TcpStream, file_folder: Option<String>) {
    let mut buf = [0u8; 4096];
    match stream.read(&mut buf).await {
        Ok(n) => {
            let tmp = String::from_utf8(buf[..n].to_vec());
            println!("{:?}", tmp);
            let Some(request) = Request::from_buffer(&buf[..n]) else {
                return;
            };
            match (request.verb, request.target.as_str()) {
                (Verb::GET, "/") => stream
                    .write_all(&Response::new(Status::OK).as_bytes())
                    .await
                    .unwrap(),
                (Verb::GET, target) if target.starts_with("/echo/") => {
                    let echo_back = request.target.strip_prefix("/echo/").unwrap();
                    let mut response = Response::new(Status::OK);
                    response.set_body(response::Content::Text(echo_back.to_owned()));
                    stream.write_all(&response.as_bytes()).await.unwrap();
                }
                (Verb::GET, "/user-agent") => {
                    let Some(user_agent) = request.headers.get("User-Agent") else {
                        stream
                            .write_all(&Response::new(Status::NotFound).as_bytes())
                            .await
                            .unwrap();
                        return;
                    };
                    let mut response = Response::new(Status::OK);
                    response.set_body(response::Content::Text(user_agent.to_owned()));
                    stream.write_all(&response.as_bytes()).await.unwrap();
                }
                (Verb::GET, target) if target.starts_with("/files/") => {
                    let Some(folder) = file_folder else {
                        return;
                    };
                    let file = target
                        .strip_prefix("/files/")
                        .map(|s| s.to_owned())
                        .unwrap();
                    let file_path = [folder, file].iter().collect::<PathBuf>();
                    let Ok(file) = fs::read(file_path) else {
                        stream
                            .write_all(&Response::new(Status::NotFound).as_bytes())
                            .await
                            .unwrap();
                        return;
                    };
                    let mut response = Response::new(Status::OK);
                    response.set_body(response::Content::OctetStream(file));
                    stream.write_all(&response.as_bytes()).await.unwrap();
                }
                (Verb::POST, target) if target.starts_with("/files/") => {
                    let Some(folder) = file_folder else {
                        return;
                    };
                    let filename = target.strip_prefix("/files/").unwrap().to_string();
                    if let Some(Content::OctetStream(content)) = request.content {
                        let path = [folder, filename].iter().collect::<PathBuf>();
                        fs::write(path, content).unwrap();
                        stream
                            .write_all(&Response::new(Status::Created).as_bytes())
                            .await
                            .unwrap();
                    }
                }
                _ => stream
                    .write_all(&Response::new(Status::NotFound).as_bytes())
                    .await
                    .unwrap(),
            };

            println!("accepted new connection");
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
}

fn parse_args(args_list: Vec<String>) -> HashMap<String, String> {
    let mut args = HashMap::<String, String>::new();

    let mut args_iter = args_list.into_iter();

    // Pop first element (program path)
    let _ = args_iter.next();

    // Iterate over pairs of (--option, value)
    while let Some(option) = args_iter.next() {
        let Some(option) = option.strip_prefix("--").map(|s| s.to_string()) else {
            break;
        };
        let Some(value) = args_iter.next() else {
            println!("No value found for option {option}, passing...");
            break;
        };
        args.insert(option, value);
    }

    args
}
