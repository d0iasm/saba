extern crate alloc;

use net_std as net;
use ui_cui as ui;

use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use net::http::HttpClient;
use saba_core::browser::Browser;
use saba_core::error::Error;
use saba_core::http::HttpResponse;
use saba_core::url::HtmlUrl;
use ui::app::Tui;

fn handle_url(url: String) -> Result<HttpResponse, Error> {
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

    // send a HTTP request and get a response
    let client = HttpClient::new();
    let response = match client.get(
        parsed_url.host(),
        parsed_url
            .port()
            .parse::<u16>()
            .unwrap_or_else(|_| panic!("port number should be u16 but got {}", parsed_url.port())),
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
                match redirect_client.get(
                    redirect_parsed_url.host(),
                    redirect_parsed_url
                        .port()
                        .parse::<u16>()
                        .unwrap_or_else(|_| {
                            panic!("port number should be u16 but got {}", parsed_url.port())
                        }),
                    redirect_parsed_url.path(),
                ) {
                    Ok(res) => res,
                    Err(e) => return Err(Error::Network(format!("{:?}", e))),
                }
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

    Ok(response)
}

fn main() {
    // initialize the UI object
    let ui = Rc::new(RefCell::new(Tui::new()));

    // initialize the main browesr struct
    let browser = Browser::new();
    ui.borrow_mut().set_browser(Rc::downgrade(&browser));

    match ui.borrow_mut().start(handle_url) {
        Ok(_) => {}
        Err(e) => {
            println!("browser fails to start {:?}", e);
        }
    };
}
