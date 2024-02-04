extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use noli::net::lookup_host;
use noli::net::IpV4Addr;
use noli::net::SocketAddr;
use noli::net::TcpStream;
use noli::print::hexdump;
use noli::println;
use toybr_core::error::Error;
use toybr_core::http::HttpResponse;

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        let ips = lookup_host(&"example.com").unwrap();

        let socket_addr: SocketAddr = (ips[0], port).into();
        println!("socket_addr: {:?}", socket_addr);

        let mut stream = TcpStream::connect(socket_addr).unwrap();

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

        println!("http request: {:?}", request);

        let bytes_written = stream.write(request.as_bytes()).unwrap();

        let mut buf = [0u8; 40960];
        let bytes_read = stream.read(&mut buf).unwrap();
        let data = &buf[..bytes_read];

        let raw_response = core::str::from_utf8(data).unwrap();

        println!("raw_response: {:?}", raw_response);

        HttpResponse::new(raw_response.to_string())
    }

    pub fn post(&self) {}
    pub fn put(&self) {}
    pub fn delete(&self) {}
}
