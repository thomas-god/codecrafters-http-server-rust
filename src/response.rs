use std::io::Write;

use flate2::{write::GzEncoder, Compression};

#[derive(Clone, Copy)]
pub enum Status {
    OK = 200,
    Created = 201,
    NotFound = 404,
}

impl Default for Status {
    fn default() -> Self {
        Self::OK
    }
}

impl Status {
    pub fn as_string(&self) -> String {
        match self {
            Self::OK => "OK".to_string(),
            Self::Created => "Created".to_string(),
            Self::NotFound => "Not Found".to_string(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Content {
    Text(String),
    OctetStream(Vec<u8>),
}

pub enum CompressionAlgorithms {
    GZIP,
}

pub struct Response {
    pub status: Status,
    pub body: Option<Content>,
    pub compression: Option<CompressionAlgorithms>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            status: Status::default(),
            body: None,
            compression: None,
        }
    }

    pub fn set_body(&mut self, body: Content) {
        self.body = Some(body);
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn set_compression(&mut self, compression: CompressionAlgorithms) {
        self.compression = Some(compression);
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        // Write status line
        let status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status as u16,
            self.status.as_string()
        );
        bytes.extend_from_slice(status_line.as_bytes());

        // Write headers and content
        let mut headers = String::new();
        let mut content_bytes: Vec<u8> = Vec::new();

        match (&self.body, &self.compression) {
            (Some(Content::Text(content)), None) => {
                headers.push_str("Content-Type: text/plain\r\n");
                headers.push_str(&format!("Content-Length: {}\r\n", content.as_bytes().len()));
                content_bytes.extend_from_slice(content.as_bytes());
            }
            (Some(Content::Text(content)), Some(CompressionAlgorithms::GZIP)) => {
                headers.push_str("Content-Encoding: gzip\r\n");
                headers.push_str("Content-Type: text/plain\r\n");
                let mut compressed_buf = GzEncoder::new(Vec::new(), Compression::default());
                compressed_buf.write_all(content.as_bytes()).unwrap();
                content_bytes.extend_from_slice(&compressed_buf.finish().unwrap());
                headers.push_str(&format!("Content-Length: {}\r\n", content_bytes.len()));
            }
            (Some(Content::OctetStream(bytes)), None) => {
                headers.push_str("Content-Type: application/octet-stream\r\n");
                headers.push_str(&format!("Content-Length: {}\r\n", bytes.len()));
                content_bytes.extend_from_slice(bytes);
            }
            _ => {}
        };

        // Empty line to signify headers part's end
        headers.push_str("\r\n");

        // Write headers and content
        bytes.extend_from_slice(headers.as_bytes());
        bytes.extend_from_slice(&content_bytes);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use crate::response::{CompressionAlgorithms, Content, Response, Status};

    #[test]
    fn test_response_as_string() {
        let mut response = Response::new();
        response.set_status(Status::OK);
        assert_eq!(
            Response::new().as_bytes(),
            "HTTP/1.1 200 OK\r\n\r\n".to_owned().as_bytes()
        );

        let mut response = Response::new();
        response.set_status(Status::NotFound);
        assert_eq!(
            response.as_bytes(),
            "HTTP/1.1 404 Not Found\r\n\r\n".to_owned().as_bytes()
        );

        let mut res = Response::new();
        res.set_status(Status::OK);
        res.set_body(Content::Text("toto".to_owned()));
        assert_eq!(
            res.as_bytes(),
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\n\r\ntoto"
                .to_owned()
                .as_bytes()
        );

        let mut res = Response::new();
        res.set_status(Status::OK);
        res.set_body(Content::OctetStream("toto".as_bytes().to_vec()));
        assert_eq!(
            res.as_bytes(),
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 4\r\n\r\ntoto"
                .to_owned()
                .as_bytes()
        )
    }
}
