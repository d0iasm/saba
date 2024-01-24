extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use noli::net::*;
use toybr_core::error::Error;
use toybr_core::http::HttpResponse;

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self) -> Result<HttpResponse, Error> {
        Err(Error::Network("failed to get HTTP response".to_string()))
    }

    pub fn post(&self) {}
    pub fn put(&self) {}
    pub fn delete(&self) {}

    fn parse_response(raw_response: String) -> Result<HttpResponse, Error> {
        Err(Error::Network("failed to parse HTTP response".to_string()))
    }
}
