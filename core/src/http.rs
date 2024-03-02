use crate::alloc::string::ToString;
use crate::error::Error;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    _version: String,
    status_code: u32,
    _reason: String,
    headers: Vec<Header>,
    body: String,
}

impl HttpResponse {
    pub fn new(raw_response: String) -> Result<Self, Error> {
        let preprocessed_response = raw_response.trim().replace("\n\r", "\n");

        let (status_line, remaining) = match preprocessed_response.split_once('\n') {
            Some((s, r)) => (s, r),
            None => panic!("http response doesn't have a new line"),
        };

        let (headers, body) = match remaining.split_once("\n\n") {
            Some((h, b)) => {
                let mut headers = Vec::new();
                for header in h.split('\n') {
                    // TODO: remove a new line cleaned_header
                    let cleaned_header = header.replace('\r', "");
                    let splitted_header: Vec<&str> = cleaned_header.splitn(2, ':').collect();

                    headers.push(Header::new(
                        String::from(splitted_header[0]),
                        // TODO: remove a whitespace correctly
                        splitted_header[1].replacen(' ', "", 1),
                    ));
                }
                (headers, b)
            }
            None => (Vec::new(), remaining),
        };

        let statuses: Vec<&str> = status_line.split(' ').collect();

        Ok(Self {
            _version: statuses[0].to_string(),
            status_code: statuses[1].parse().unwrap_or(404),
            _reason: statuses[2].to_string(),
            headers,
            body: body.to_string(),
        })
    }
    pub fn status_code(&self) -> u32 {
        self.status_code
    }

    pub fn body(&self) -> String {
        self.body.clone()
    }

    pub fn header(&self, name: &str) -> String {
        for h in &self.headers {
            if h.name == name {
                return h.value.clone();
            }
        }

        // TODO: return None
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_line_only() {
        //let raw = "HTTP/1.1 200 OK".to_string();
        //let res = HttpResponse::new(raw);
        //assert_eq!(&res.version(), "HTTP/1.1");
        //assert_eq!(&res.status_code(), 200);
        //assert_eq!(&res.reason(), "OK");
    }
}
