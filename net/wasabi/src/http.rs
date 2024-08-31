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
use noli::net::IpV4Addr;
use noli::net::SocketAddr;
use noli::net::TcpStream;
use saba_core::error::Error;
use saba_core::http::HttpResponse;

static FAKE_RESPONSE_BODY: &str = r#"<!doctype html>
<html>
<head>
    <title>Example Domain title</title>

    <meta charset="utf-8" />
    <meta http-equiv="Content-type" content="text/html; charset=utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style type="text/css">
    body {
        background-color: #f0f0f2;
        margin: 0;
        padding: 0;
        font-family: -apple-system, system-ui, BlinkMacSystemFont, "Segoe UI", "Open Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;

    }
    div {
        width: 600px;
        margin: 5em auto;
        padding: 2em;
        background-color: #fdfdff;
        border-radius: 0.5em;
        box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);
    }
    a:link, a:visited {
        color: #38488f;
        text-decoration: none;
    }
    @media (max-width: 700px) {
        div {
            margin: 0 auto;
            width: auto;
        }
    }
    </style>
</head>
<body>
<div>
    <h1>Example Domain</h1>
    <p>This domain is for use in illustrative examples in documents. You may use this
    domain in literature without prior coordination or asking for permission.</p>
    <p><a href="https://www.iana.org/domains/example">More information...</a></p>
</div>
</body>
</html>
"#;

/*
static FAKE_RESPONSE_BODY: &str = r#"<html>
<head>
    <title>Example Domain Response</title>
    <meta charset="utf-8" />
    <style>
    #title {
      color: red;
    }
    .first {
      color: #0000ff;
    }
    .hidden {
      display: none;
    }
    .link {
      background-color: #00ffff;
    }
    </style>
    <script type="text/javascript">
      var target=document.getElementById("target");
      console.log(target);
      function foo(a, b) {
        return a+b;
      }
      target.textContent=foo(1, 2) + 3;
      target.textContent="dynamic text";
     </script>
</head>
<body>
    <h1 id="title">Example Domain Response</h1>
    <p class="first">This domain is for use in illustrative examples in documents. You may use this
    domain in literature without prior coordination or asking for permission.</p>
    <div class="link">
      <p class="link-wrapper"><a>Link1</a><a>Link2</a></p>
    </div>
    <p id="target">original text</p>
    <p class="wrapper"><p class="text1">Text1</p><p class="text2">Text2</p></p>
    <p><a href="https://www.iana.org/domains/example">More information...</a></p>
    <p class="hidden">none</p>
    <img src="https://placehold.co/600x400"/>
</body>
</html>
"#;
*/

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        /*
        let ips = match lookup_host(&"example.com") {
            Ok(ips) => ips,
            Err(_) => return Err(Error::Network("Failed to find IP addresses".to_string())),
        };
        */
        // TODO: Remove this temporary IP address.
        let ips = [IpV4Addr::new([0, 0, 0, 0])];

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

        let raw_response = match core::str::from_utf8(data) {
            Ok(r) => r,
            Err(_) => return Err(Error::Network("Invalid UTF8 data".to_string())),
        };

        //println!("raw_response: {:?}", raw_response);

        //TODO: remove FAKE_RESPONSE_BODY
        match HttpResponse::new(raw_response.to_string()) {
            Ok(mut res) => {
                res.body = FAKE_RESPONSE_BODY.to_string();
                Ok(res)
            }
            Err(e) => Err(e),
        }
    }

    pub fn post(&self) {}
    pub fn put(&self) {}
    pub fn delete(&self) {}
}
