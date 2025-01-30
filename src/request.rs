use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Verb {
    GET,
}

impl Verb {
    pub fn from_str(word: &str) -> Option<Verb> {
        match word {
            "GET" => Some(Verb::GET),
            _ => None,
        }
    }
}
pub struct Request {
    pub verb: Verb,
    pub target: String,
    pub headers: HashMap<String, String>,
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
        for line in iter {
            if line == "" {
                break;
            }
            let mut header_iter = line.split(": ");
            let (Some(header), Some(value)) = (header_iter.next(), header_iter.next()) else {
                break;
            };
            headers.insert(header.to_string(), value.to_string());
        }

        Some(Request {
            verb,
            target: target.to_string(),
            headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::request::Verb;

    use super::Request;

    #[test]
    fn test_parse_request() {
        let buffer = "GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n".as_bytes();

        let request = Request::from_buffer(&buffer);
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
    }
}
