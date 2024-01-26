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

static SAMPLE_HTTP_GET_REQUEST: &str = "
GET / HTTP/1.1
Host: example.com

";

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        let ip_addr = IpV4Addr::new([127, 0, 0, 1]);
        let port = 80;
        let socket_addr: SocketAddr = (ip_addr, port).into();
        let mut stream = TcpStream::connect(socket_addr).unwrap();
        let bytes_written = stream.write(SAMPLE_HTTP_GET_REQUEST.as_bytes()).unwrap();
        println!("bytes_written = {bytes_written}");
        let mut buf = [0u8; 4096];
        let bytes_read = stream.read(&mut buf).unwrap();
        let data = &buf[..bytes_read];
        println!("bytes_read = {bytes_read}");
        hexdump(data);

        let raw_response = core::str::from_utf8(data).unwrap();

        println!("raw_response: {:?}", raw_response);

        HttpResponse::new(raw_response.to_string())

        /*
        Ok(HttpResponse {
            _version: "1.1".to_string(),
            status_code: 200,
            _reason: "".to_string(),
            headers: Vec::new(),
            body: r#"
        <html>
        <head>
          <style type="text/css">
            div {
              width: 600;
              margin: 200;
              background-color: white;
            }
          </style>
        </head>
        <body>
        <div>
          <h1>Example Domain</h1>
          <p>This domain is for use in illustrative examples in documents. You may use this
          domain in literature without prior coordination or asking for permission.</p>
          <p><a href="http://localhost:8000/test2.html">More information...</a></p>
        </div>
        </body>
        </html>"#
                .to_string(),
        })
        */
    }

    pub fn get2(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        let ips = lookup_host(&"example.com").unwrap();

        let socket_addr: SocketAddr = (ips[0], port).into();
        println!("socket_addr: {:?}", socket_addr);

        let mut stream = TcpStream::connect(socket_addr).unwrap();

        let mut request = String::from("\nGET /");
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

        //let bytes_written = stream.write(request.as_bytes()).unwrap();
        let bytes_written = stream.write(SAMPLE_HTTP_GET_REQUEST.as_bytes()).unwrap();

        println!("bytes_written: {:?}", bytes_written);

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
