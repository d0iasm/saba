//! RFC 1738 - Uniform Resource Locators (URL): https://datatracker.ietf.org/doc/html/rfc1738
//! RFC 3986 - Uniform Resource Identifier (URI): https://datatracker.ietf.org/doc/html/rfc3986

use std::string::String;
use std::string::ToString;
use std::vec::Vec;

#[derive(Debug)]
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

    fn default_port_number(&self) -> u16 {
        match self {
            Protocol::Http => 80,
            Protocol::Https => 443,
        }
    }
}

#[derive(Debug)]
pub struct ParsedUrl {
    _scheme: Protocol,
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl ParsedUrl {
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
}
