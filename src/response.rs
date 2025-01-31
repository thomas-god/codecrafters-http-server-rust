#[derive(Clone, Copy)]
pub enum Status {
    OK = 200,
    NotFound = 404,
}

impl Status {
    pub fn as_string(&self) -> String {
        match self {
            Self::OK => "OK".to_string(),
            Self::NotFound => "Not Found".to_string(),
        }
    }
}

pub enum Content {
    Text(String),
    OctetStream(Vec<u8>),
}

pub struct Response {
    pub status: Status,
    pub body: Option<Content>,
}

impl Response {
    pub fn new(status: Status) -> Response {
        Response { status, body: None }
    }

    pub fn set_body(&mut self, body: Content) {
        self.body = Some(body);
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status as u16,
            self.status.as_string()
        );
        let mut headers = String::new();
        let body = match &self.body {
            Some(Content::Text(content)) => {
                headers.push_str("Content-Type: text/plain\r\n");
                headers.push_str(&format!("Content-Length: {}\r\n", content.as_bytes().len()));
                content.as_bytes()
            }
            Some(Content::OctetStream(bytes)) => {
                headers.push_str("Content-Type: application/octet-stream\r\n");
                headers.push_str(&format!("Content-Length: {}\r\n", bytes.len()));
                bytes
            }
            _ => &[],
        };
        headers.push_str("\r\n");

        let mut bytes = Vec::<u8>::new();
        bytes.extend_from_slice(status_line.as_bytes());
        bytes.extend_from_slice(headers.as_bytes());
        bytes.extend_from_slice(body);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use crate::response::{Content, Response, Status};

    #[test]
    fn test_response_as_string() {
        assert_eq!(
            Response::new(Status::OK).as_bytes(),
            "HTTP/1.1 200 OK\r\n\r\n".to_owned().as_bytes()
        );
        assert_eq!(
            Response::new(Status::NotFound).as_bytes(),
            "HTTP/1.1 404 Not Found\r\n\r\n".to_owned().as_bytes()
        );

        let mut res = Response::new(Status::OK);
        res.set_body(Content::Text("toto".to_owned()));
        assert_eq!(
            res.as_bytes(),
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\n\r\ntoto"
                .to_owned()
                .as_bytes()
        );

        let mut res = Response::new(Status::OK);
        res.set_body(Content::OctetStream("toto".as_bytes().to_vec()));
        assert_eq!(
            res.as_bytes(),
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 4\r\n\r\ntoto"
                .to_owned()
                .as_bytes()
        )
    }
}
