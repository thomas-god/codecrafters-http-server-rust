use std::collections::HashMap;

use crate::response::Content;

#[derive(Debug, PartialEq)]
pub enum Verb {
    GET,
    POST,
}

impl Verb {
    pub fn from_str(word: &str) -> Option<Verb> {
        match word {
            "GET" => Some(Verb::GET),
            "POST" => Some(Verb::POST),
            _ => None,
        }
    }
}
pub struct Request {
    pub verb: Verb,
    pub target: String,
    pub headers: HashMap<String, String>,
    pub content: Option<Content>,
}

impl Request {
    pub fn from_buffer(buffer: &[u8]) -> Option<Request> {
        let content = String::from_utf8(buffer.to_vec()).ok()?;

        let mut iter = content.split("\r\n");

        // Parse header line <method> <request-target> <protocol>
        let mut request_line = iter.next().map(|line| line.split(" "))?;
        let verb = request_line.next().and_then(Verb::from_str)?;
        let target = request_line.next()?;

        // Parse headers
        let mut headers = HashMap::<String, String>::new();
        for line in iter.by_ref() {
            if line.is_empty() {
                // Empty line means end of headers part
                break;
            }
            let mut header_iter = line.split(": ");
            let (Some(header), Some(value)) = (header_iter.next(), header_iter.next()) else {
                break;
            };
            headers.insert(header.to_string(), value.to_string());
        }

        // Parse content if any
        let content = match iter.next() {
            Some(content) => match headers.get("Content-Type").map(|s| s.as_str()) {
                Some("text/plain") => Some(Content::Text(content.to_owned())),
                Some("application/octet-stream") => {
                    Some(Content::OctetStream(content.as_bytes().to_vec()))
                }
                _ => None,
            },
            _ => None,
        };

        println!("{verb:?}, {headers:?}, {content:?}");
        Some(Request {
            verb,
            target: target.to_string(),
            headers,
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{request::Verb, response::Content};

    use super::Request;

    #[test]
    fn test_parse_request() {
        let buffer = "GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n".as_bytes();

        let request = Request::from_buffer(buffer);
        assert!(request.is_some());

        let request = request.unwrap();
        assert_eq!(request.verb, Verb::GET);
        assert_eq!(request.target, "/index.html".to_string());
        assert_eq!(
            request.headers,
            HashMap::from([
                ("Host".to_string(), "localhost:4221".to_string()),
                ("User-Agent".to_string(), "curl/7.64.1".to_string()),
                ("Accept".to_string(), "*/*".to_string())
            ])
        );
        assert!(request.content.is_none());
    }

    #[test]
    fn test_parse_request_with_body() {
        let buffer ="POST /files/toto.txt HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Type: application/octet-stream\r\nContent-Length: 5\r\n\r\n12345".as_bytes();

        let request = Request::from_buffer(buffer);
        // assert!(request.is_some());

        let request = request.unwrap();

        assert_eq!(request.verb, Verb::POST);
        assert_eq!(request.target, "/files/toto.txt".to_owned());
        assert_eq!(
            request.headers,
            HashMap::from([
                ("Host".to_string(), "localhost:4221".to_string()),
                ("User-Agent".to_string(), "curl/7.64.1".to_string()),
                ("Accept".to_string(), "*/*".to_string()),
                (
                    "Content-Type".to_string(),
                    "application/octet-stream".to_string()
                ),
                ("Content-Length".to_string(), "5".to_string())
            ])
        );
        assert_eq!(
            request.content,
            Some(Content::OctetStream("12345".as_bytes().to_vec()))
        )
    }
}
