//! RFC 1738 - Uniform Resource Locators (URL): https://datatracker.ietf.org/doc/html/rfc1738
//! RFC 3986 - Uniform Resource Identifier (URI): https://datatracker.ietf.org/doc/html/rfc3986

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUrl {
    scheme: String,
    host: String,
    port: String,
    path: String,
}

impl ParsedUrl {
    pub fn new(url: String) -> Self {
        // HTTP format
        // http://<host>:<port>/<path>?<searchpart>
        //
        // https://datatracker.ietf.org/doc/html/rfc1738#section-3.3
        //
        // possible format:
        // https://url.spec.whatwg.org/#urls

        let url_parts: Vec<&str> = url.trim_start_matches("http://").splitn(2, "/").collect();

        let path;
        if url_parts.len() < 2 {
            path = "".to_string()
        } else {
            path = url_parts[1].to_string();
        }

        let host_and_port = url_parts[0];
        let host;
        let port;
        if let Some(index) = host_and_port.find(':') {
            host = &host_and_port[..index];
            port = &host_and_port[index + 1..];
        } else {
            host = host_and_port;
            // 80 is the default port number of HTTP scheme.
            // Default port numbers are defined by Internet Assigned Numbers Authority (IANA).
            // https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml
            port = "80";
        }

        Self {
            // TODO: currently, support only HTTP scheme.
            scheme: "http".to_string(),
            host: host.to_string(),
            port: port.to_string(),
            path: path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url1() {
        let url = "http://example.com".to_string();
        let expected = ParsedUrl {
            scheme: "http".to_string(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }

    #[test]
    fn test_url2() {
        let url = "http://example.com:8888".to_string();
        let expected = ParsedUrl {
            scheme: "http".to_string(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }

    #[test]
    fn test_url3() {
        let url = "http://example.com:8888/index.html".to_string();
        let expected = ParsedUrl {
            scheme: "http".to_string(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }

    #[test]
    fn test_url4() {
        let url = "example.com:8888/index.html".to_string();
        let expected = ParsedUrl {
            scheme: "http".to_string(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }

    #[test]
    fn test_localhost() {
        let url = "localhost:8000".to_string();
        let expected = ParsedUrl {
            scheme: "http".to_string(),
            host: "localhost".to_string(),
            port: "8000".to_string(),
            path: "".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }

    /*
    #[test]
    fn test_unsupported_url() {
        let url = "https://example.com:8888/index.html".to_string();
        let expected = ParsedUrl {
            scheme: "https".to_string(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }
    */
}
