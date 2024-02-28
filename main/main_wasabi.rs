#![no_std]
#![no_main]

extern crate alloc;

use crate::alloc::string::ToString;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use net_wasabi::http::HttpClient;
use noli::*;
use saba_core::browser::Browser;
use saba_core::error::Error;
use saba_core::http::HttpResponse;
use saba_core::renderer::page::Page;
use saba_core::ui::UiObject;
use saba_core::url::HtmlUrl;
use ui_wasabi::WasabiUI;

fn handle_url<U: UiObject>(url: String) -> Result<HttpResponse, Error> {
    println!("handle_url");

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
                let redirect_parsed_url = HtmlUrl::new(res.header("Location"));

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
    // initialize the UI object
    let ui = Rc::new(RefCell::new(WasabiUI::new()));
    let page = Rc::new(RefCell::new(Page::new()));

    // initialize the main browesr struct
    let browser = Rc::new(RefCell::new(Browser::new(ui.clone(), page.clone())));
    ui.borrow_mut().set_browser(Rc::downgrade(&browser));
    page.borrow_mut().set_browser(Rc::downgrade(&browser));

    match ui.borrow_mut().start(handle_url::<WasabiUI>) {
        Ok(_) => {}
        Err(e) => {
            println!("browser fails to start {:?}", e);
            return 1;
        }
    };

    0
}

entry_point!(main);
