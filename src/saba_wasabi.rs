#![no_std]
#![no_main]

extern crate alloc;

#[cfg_attr(target_os = "linux", no_main)]
use noli::prelude::*;
entry_point!(main);

use crate::alloc::string::ToString;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use net_wasabi::http::HttpClient;
use noli::println;
use saba_core::browser::Browser;
use saba_core::error::Error;
use saba_core::http::HttpResponse;
use saba_core::url::HtmlUrl;
use ui_wasabi::app::WasabiUI;

fn handle_url(url: String) -> core::result::Result<HttpResponse, Error> {
    // parse url
    let parsed_url = match HtmlUrl::new(url.to_string()).parse() {
        Ok(url) => url,
        Err(e) => {
            return Err(Error::UnexpectedInput(format!(
                "input html is not supported: {:?}",
                e
            )));
        }
    };
    println!("parsed_url: {:?}", parsed_url);

    // send a HTTP request and get a response
    let client = HttpClient::new();
    let response = match client.get(
        parsed_url.host(),
        parsed_url.port().parse::<u16>().expect(&format!(
            "port number should be u16 but got {}",
            parsed_url.port()
        )),
        parsed_url.path(),
    ) {
        Ok(res) => {
            // redirect to Location
            if res.status_code() == 302 {
                let location = match res.header_value("Location") {
                    Ok(value) => value,
                    Err(_) => return Ok(res),
                };
                let redirect_parsed_url = HtmlUrl::new(location);

                let redirect_client = HttpClient::new();
                let redirect_res = match redirect_client.get(
                    redirect_parsed_url.host(),
                    redirect_parsed_url.port().parse::<u16>().expect(&format!(
                        "port number should be u16 but got {}",
                        parsed_url.port()
                    )),
                    redirect_parsed_url.path(),
                ) {
                    Ok(res) => res,
                    Err(e) => return Err(Error::Network(format!("{:?}", e))),
                };

                redirect_res
            } else {
                res
            }
        }
        Err(e) => {
            return Err(Error::Network(format!(
                "failed to get http response: {:?}",
                e
            )))
        }
    };

    println!("response: {:?}", response);

    Ok(response)
}

fn main() -> u64 {
    // initialize the main browesr struct
    let browser = Browser::new();

    // initialize the UI object
    let ui = Rc::new(RefCell::new(WasabiUI::new(browser)));

    match ui.borrow_mut().start(handle_url) {
        Ok(_) => {}
        Err(e) => {
            println!("browser fails to start {:?}", e);
            return 1;
        }
    };

    0
}
