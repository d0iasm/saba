//! Http client aligned with a subset of HTTP/1.1.
//!
//! https://tools.ietf.org/html/rfc7230
//! https://tools.ietf.org/html/rfc7231
//! https://tools.ietf.org/html/rfc7232
//! https://tools.ietf.org/html/rfc7233
//! https://tools.ietf.org/html/rfc7234
//! https://tools.ietf.org/html/rfc7235

use dns_lookup::lookup_host;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::io::Read;
use std::net::TcpStream;
use std::string::String;
use std::vec::Vec;
use toybr_core::http::HttpResponse;

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> std::io::Result<HttpResponse> {
        let ips = lookup_host(&host)?.into_iter();
        let ipv4s: Vec<std::net::IpAddr> = ips.filter(|ip| ip.is_ipv4()).collect();

        let mut stream = TcpStream::connect((ipv4s[0], port))?;

        let mut request = String::from("GET /");
        request.push_str(&path);
        request.push_str(" HTTP/1.1\n");

        // headers
        request.push_str("Host: ");
        request.push_str(&host);
        request.push('\n');
        request.push_str("Accept: */*\n");
        request.push_str("Connection: close\n");

        request.push('\n');

        stream.write(request.as_bytes())?;

        let mut buf = String::new();
        stream.read_to_string(&mut buf)?;

        match HttpResponse::new(buf.to_string()) {
            Ok(res) => Ok(res),
            Err(e) => Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("{:?}", e),
            )),
        }
    }

    // TODO: support correctly
    /*
        pub fn _post(&self, url: &ParsedUrl, _body: String) -> std::io::Result<HttpResponse> {
            let ips: Vec<std::net::IpAddr> = lookup_host(&url.host)?;

            let mut stream = TcpStream::connect((ips[0], url.port))?;

            let mut request = String::from("POST ");
            request.push_str(&url.path);
            request.push_str(" HTTP/1.1\n");

            /*
            // headers
            for h in &url.headers {
                request.push_str(&h.key);
                request.push_str(": ");
                request.push_str(&h.value);
                request.push('\n');
            }
            */

            request.push('\n');

            stream.write(request.as_bytes())?;

            let mut buf = String::new();
            stream.read_to_string(&mut buf)?;

            Ok(HttpResponse::new(buf))
        }
    */
}
