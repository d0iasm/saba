//! Http client aligned with a subset of HTTP/1.1.
//!
//! https://tools.ietf.org/html/rfc7230
//! https://tools.ietf.org/html/rfc7231
//! https://tools.ietf.org/html/rfc7232
//! https://tools.ietf.org/html/rfc7233
//! https://tools.ietf.org/html/rfc7234
//! https://tools.ietf.org/html/rfc7235

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use noli::net::lookup_host;
use noli::net::SocketAddr;
use noli::net::TcpStream;
//use noli::println;
use saba_core::error::Error;
use saba_core::http::HttpResponse;

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        let ips = match lookup_host(&"example.com") {
            Ok(ips) => ips,
            Err(_) => return Err(Error::Network("Failed to find IP addresses".to_string())),
        };

        if ips.len() < 1 {
            return Err(Error::Network("Failed to find IP addresses".to_string()));
        }

        let socket_addr: SocketAddr = (ips[0], port).into();

        let mut stream = match TcpStream::connect(socket_addr) {
            Ok(stream) => stream,
            Err(_) => {
                return Err(Error::Network(
                    "Failed to connect to TCP stream".to_string(),
                ))
            }
        };

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

        //println!("http request: {:?}", request);

        let _bytes_written = match stream.write(request.as_bytes()) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(Error::Network(
                    "Failed to send a request to TCP stream".to_string(),
                ))
            }
        };

        let mut buf = [0u8; 40960];
        let bytes_read = match stream.read(&mut buf) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(Error::Network(
                    "Failed to receive a request from TCP stream".to_string(),
                ))
            }
        };
        let data = &buf[..bytes_read];

        let raw_response = core::str::from_utf8(data).unwrap();

        //println!("raw_response: {:?}", raw_response);

        HttpResponse::new(raw_response.to_string())
    }

    pub fn post(&self) {}
    pub fn put(&self) {}
    pub fn delete(&self) {}
}
