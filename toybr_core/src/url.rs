//! RFC 1738 - Uniform Resource Locators (URL): https://datatracker.ietf.org/doc/html/rfc1738
//! RFC 3986 - Uniform Resource Identifier (URI): https://datatracker.ietf.org/doc/html/rfc3986

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/*
#[derive(Debug, Clone, PartialEq)]
enum Protocol {
    Http,
    Https,
}

impl Protocol {
    fn to_string(&self) -> String {
        match self {
            Protocol::Http => String::from("http"),
            Protocol::Https => String::from("https"),
        }
    }

    /// Default port numbers are defined by Internet Assigned Numbers Authority (IANA).
    /// https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml
    fn default_port_number(&self) -> u16 {
        match self {
            // https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml?&page=2
            Protocol::Http => 80,
            //
            Protocol::Https => 443,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUrl {
    _scheme: Protocol,
    pub host: String,
    pub port: u16,
    pub path: String,
}
*/

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUrl {
    scheme: String,
    host: String,
    port: String,
    path: String,
}

impl ParsedUrl {
    /*
    fn extract_scheme(url: &String) -> Protocol {
        let splitted_url: Vec<&str> = url.split("://").collect();
        if splitted_url.len() == 2 && splitted_url[0] == Protocol::Http.to_string() {
            Protocol::Http
        } else if splitted_url.len() == 2 && splitted_url[0] == Protocol::Https.to_string() {
            Protocol::Https
        } else if splitted_url.len() == 1 {
            // No scheme. Set "HTTP" as a default behavior.
            Protocol::Http
        } else {
            panic!("unsupported scheme: {}", url);
        }
    }

    fn remove_scheme(url: &String, scheme: &Protocol) -> String {
        // Remove "scheme://" from url if any.
        url.replacen(&(scheme.to_string() + "://"), "", 1)
    }

    fn extract_host(url: &String) -> String {
        let splitted_url: Vec<&str> = url.splitn(2, '/').collect();
        let host_and_port: Vec<&str> = splitted_url[0].splitn(2, ':').collect();
        host_and_port[0].to_string()
    }

    fn extract_path(url: &String) -> Option<String> {
        let splitted_url: Vec<&str> = url.splitn(2, '/').collect();
        if splitted_url.len() == 2 {
            Some(splitted_url[1].to_string())
        } else {
            None
        }
    }

    fn extract_port(url: &String) -> Option<u16> {
        let splitted_url: Vec<&str> = url.splitn(2, '/').collect();
        let host_and_port: Vec<&str> = splitted_url[0].splitn(2, ':').collect();
        if host_and_port.len() == 2 {
            Some(host_and_port[1].parse::<u16>().unwrap())
        } else {
            None
        }
    }

    pub fn new(original_url: String) -> Self {
        // HTTP format
        // http://<host>:<port>/<path>?<searchpart>
        //
        // https://datatracker.ietf.org/doc/html/rfc1738#section-3.3
        //
        // possible format:
        // https://url.spec.whatwg.org/#urls

        let scheme = Self::extract_scheme(&original_url);
        let url = Self::remove_scheme(&original_url, &scheme);

        let host = Self::extract_host(&url);
        let path = match Self::extract_path(&url) {
            Some(p) => p,
            None => String::new(),
        };

        let port = match Self::extract_port(&url) {
            Some(h) => h,
            None => scheme.default_port_number(),
        };

        Self {
            _scheme: scheme,
            host,
            port,
            path,
        }
    }
    */

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

    /*
    #[test]
    fn test_url1() {
        let url = "http://example.com".to_string();
        let expected = ParsedUrl {
            _scheme: Protocol::Http,
            host: "example.com".to_string(),
            port: 80,
            path: "".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }

    #[test]
    fn test_url2() {
        let url = "http://example.com:8888".to_string();
        let expected = ParsedUrl {
            _scheme: Protocol::Http,
            host: "example.com".to_string(),
            port: 8888,
            path: "".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }

    #[test]
    fn test_url3() {
        let url = "http://example.com:8888/index.html".to_string();
        let expected = ParsedUrl {
            _scheme: Protocol::Http,
            host: "example.com".to_string(),
            port: 8888,
            path: "index.html".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }

    #[test]
    fn test_url4() {
        let url = "example.com:8888/index.html".to_string();
        let expected = ParsedUrl {
            _scheme: Protocol::Http,
            host: "example.com".to_string(),
            port: 8888,
            path: "index.html".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }

    #[test]
    fn test_localhost() {
        let url = "localhost:8000".to_string();
        let expected = ParsedUrl {
            _scheme: Protocol::Http,
            host: "localhost".to_string(),
            port: 8000,
            path: "".to_string(),
        };
        assert_eq!(expected, ParsedUrl::new(url));
    }
    */

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
