use request::{Request, Verb};
use response::{Response, Status};
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

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process_stream(stream).await;
        });
    }
}

async fn process_stream(mut stream: TcpStream) {
    let mut buf = [0u8; 4096];
    match stream.read(&mut buf).await {
        Ok(n) => {
            let Some(request) = Request::from_buffer(&buf[..n]) else {
                return;
            };
            match (request.verb, request.target.as_str()) {
                (Verb::GET, "/") => stream
                    .write_all(Response::new(Status::OK).as_string().as_bytes())
                    .await
                    .unwrap(),
                (Verb::GET, target) if target.starts_with("/echo/") => {
                    let echo_back = request.target.strip_prefix("/echo/").unwrap();
                    let mut response = Response::new(Status::OK);
                    response.set_body(response::ContentType::Text, echo_back.to_owned());
                    stream
                        .write_all(response.as_string().as_bytes())
                        .await
                        .unwrap();
                }
                (Verb::GET, "/user-agent") => {
                    let Some(user_agent) = request.headers.get("User-Agent") else {
                        stream
                            .write_all(Response::new(Status::NotFound).as_string().as_bytes())
                            .await
                            .unwrap();
                        return;
                    };
                    let mut response = Response::new(Status::OK);
                    response.set_body(response::ContentType::Text, user_agent.to_owned());
                    stream
                        .write_all(response.as_string().as_bytes())
                        .await
                        .unwrap();
                }
                _ => stream
                    .write_all(Response::new(Status::NotFound).as_string().as_bytes())
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
