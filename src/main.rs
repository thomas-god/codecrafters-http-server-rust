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

            let res = Response::new();
            let mut response = match (&request.verb, request.target.as_str()) {
                (Verb::GET, "/") => get_root(&request, res),
                (Verb::GET, target) if target.starts_with("/echo/") => get_echo(&request, res),
                (Verb::GET, "/user-agent") => get_user_agent(&request, res),
                (Verb::GET, target) if target.starts_with("/files/") => {
                    get_file(&file_folder, target, res)
                }
                (Verb::POST, target) if target.starts_with("/files/") => {
                    post_file(file_folder, &request, target, res)
                }
                _ => not_found(&request, res),
            };

            // Check for compression
            if let Some(algorithms) = request.headers.get("Accept-Encoding") {
                if algorithms.contains("gzip") {
                    response.set_compression(response::Compression::GZIP);
                }
            }

            stream.write_all(&response.as_bytes()).await.unwrap();

            println!("accepted new connection");
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
}

fn get_root(_request: &Request, mut response: Response) -> Response {
    response.set_status(Status::OK);
    response
}

fn post_file(file_folder: Option<String>, request: &Request, target: &str, mut response: Response) -> Response {
    let Some(folder) = file_folder else {
        response.set_status(Status::NotFound);
        return response;
    };
    let filename = target.strip_prefix("/files/").unwrap().to_string();
    if let Some(Content::OctetStream(ref content)) = request.content {
        let path = [folder, filename].iter().collect::<PathBuf>();
        fs::write(path, content).unwrap();
        response.set_status(Status::Created);
        return response;
    }
    response.set_status(Status::NotFound);
    response
}

fn get_file(file_folder: &Option<String>, target: &str, mut response: Response) -> Response {
    let Some(ref folder) = *file_folder else {
        response.set_status(Status::NotFound);
        return response;
    };
    let file = target
        .strip_prefix("/files/")
        .map(|s| s.to_owned())
        .unwrap();
    let file_path = [folder, &file].iter().collect::<PathBuf>();
    let Ok(file) = fs::read(file_path) else {
        response.set_status(Status::NotFound);
        return response;
    };
    response.set_status(Status::OK);
    response.set_body(response::Content::OctetStream(file));
    response
}

fn get_user_agent(request: &Request, mut response: Response) -> Response {
    let Some(user_agent) = request.headers.get("User-Agent") else {
        response.set_status(Status::NotFound);
        return response;
    };
    response.set_status(Status::OK);
    response.set_body(response::Content::Text(user_agent.to_owned()));
    response
}

fn get_echo(request: &Request, mut response: Response) -> Response {
    let echo_back = request.target.strip_prefix("/echo/").unwrap();
    response.set_status(Status::OK);
    response.set_body(response::Content::Text(echo_back.to_owned()));
    response
}

fn not_found(_request: &Request, mut response: Response) -> Response {
    response.set_status(Status::NotFound);
    response
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
