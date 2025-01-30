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

pub enum ContentType {
    Text,
}

pub struct Response {
    pub status: Status,
    pub content_type: Option<ContentType>,
    pub body: Option<String>,
}

impl Response {
    pub fn new(status: Status) -> Response {
        Response {
            status,
            content_type: None,
            body: None,
        }
    }

    pub fn set_body(&mut self, content_type: ContentType, body: String) {
        self.content_type = Some(content_type);
        self.body = Some(body);
    }

    pub fn as_string(&self) -> String {
        let status_line = format!(
            "HTTP/1.1 {} {}",
            self.status as u16,
            self.status.as_string()
        );
        let mut headers = String::new();
        let mut body = String::new();
        if let (Some(_content_type), Some(_body)) = (&self.content_type, &self.body) {
            headers.push_str("Content-Type: text/plain\r\n");
            headers.push_str(&format!("Content-Length: {}\r\n", _body.as_bytes().len()));
            body = _body.to_owned();
        }

        [status_line, headers, body].join("\r\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::response::{ContentType, Response, Status};

    #[test]
    fn test_response_as_string() {
        assert_eq!(
            Response::new(Status::OK).as_string(),
            "HTTP/1.1 200 OK\r\n\r\n".to_owned()
        );
        assert_eq!(
            Response::new(Status::NotFound).as_string(),
            "HTTP/1.1 404 Not Found\r\n\r\n".to_owned()
        );

        let mut res = Response::new(Status::OK);
        res.set_body(ContentType::Text, "toto".to_owned());
        assert_eq!(
            res.as_string(),
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\n\r\ntoto"
                .to_owned()
        );
    }
}
