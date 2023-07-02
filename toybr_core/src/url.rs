//! RFC 1738 - Uniform Resource Locators (URL): https://datatracker.ietf.org/doc/html/rfc1738
//! This module only supports HTTP URL scheme defined at RFC 1738 section 3.3.
//! https://datatracker.ietf.org/doc/html/rfc1738#section-3.3

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// The HTTP URL scheme is used to designate Internet resources accessible using HTTP (HyperText Transfer Protocol).
/// http://<host>:<port>/<path>?<searchpart>
/// https://datatracker.ietf.org/doc/html/rfc1738#section-3.3
#[derive(Debug, Clone, PartialEq)]
pub struct HtmlUrl {
    host: String,
    port: String,
    path: String,
    searchpart: String,
}

impl HtmlUrl {
    pub fn new(url: String) -> Self {
        let url_parts: Vec<&str> = url.trim_start_matches("http://").splitn(2, "/").collect();

        let path;
        let searchpart;
        if url_parts.len() < 2 {
            // There is no path and searchpart in URL.
            path = "".to_string();
            searchpart = "".to_string();
        } else {
            let path_and_searchpart: Vec<&str> = url_parts[1].splitn(2, "?").collect();
            path = path_and_searchpart[0].to_string();
            if path_and_searchpart.len() < 2 {
                searchpart = "".to_string();
            } else {
                searchpart = path_and_searchpart[1].to_string();
            }
        }

        let host_and_port = url_parts[0];
        let host;
        let port;
        if let Some(index) = host_and_port.find(':') {
            host = host_and_port[..index].to_string();
            port = host_and_port[index + 1..].to_string();
        } else {
            host = host_and_port.to_string();
            // 80 is the default port number of HTTP scheme.
            // Default port numbers are defined by Internet Assigned Numbers Authority (IANA).
            // https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml
            port = "80".to_string();
        }

        Self {
            host,
            port,
            path,
            searchpart,
        }
    }

    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn port(&self) -> String {
        self.port.clone()
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn searchpart(&self) -> String {
        self.searchpart.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url1() {
        let url = "http://example.com".to_string();
        let expected = HtmlUrl {
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        };
        assert_eq!(expected, HtmlUrl::new(url));
    }

    #[test]
    fn test_url2() {
        let url = "http://example.com:8888".to_string();
        let expected = HtmlUrl {
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        };
        assert_eq!(expected, HtmlUrl::new(url));
    }

    #[test]
    fn test_url3() {
        let url = "http://example.com:8888/index.html".to_string();
        let expected = HtmlUrl {
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            searchpart: "".to_string(),
        };
        assert_eq!(expected, HtmlUrl::new(url));
    }

    #[test]
    fn test_url4() {
        let url = "example.com:8888/index.html".to_string();
        let expected = HtmlUrl {
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            searchpart: "".to_string(),
        };
        assert_eq!(expected, HtmlUrl::new(url));
    }

    #[test]
    fn test_url5() {
        let url = "http://example.com:8888/index.html?a=123&b=456".to_string();
        let expected = HtmlUrl {
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            searchpart: "a=123&b=456".to_string(),
        };
        assert_eq!(expected, HtmlUrl::new(url));
    }

    #[test]
    fn test_localhost() {
        let url = "localhost:8000".to_string();
        let expected = HtmlUrl {
            host: "localhost".to_string(),
            port: "8000".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        };
        assert_eq!(expected, HtmlUrl::new(url));
    }

    /*
    #[test]
    fn test_unsupported_url() {
        let url = "https://example.com:8888/index.html".to_string();
        let expected = HtmlUrl {
            scheme: "https".to_string(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
        };
        assert_eq!(expected, HtmlUrl::new(url));
    }
    */
}
