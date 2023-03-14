extern crate alloc;

use alloc::rc::Rc;
use alloc::string::String;
use browser::Browser;
use common::error::Error;
use common::ui::UiObject;
use core::cell::RefCell;
use net::http::HttpClient;
use net::http::HttpResponse;
use ui::app::Tui;
use url::ParsedUrl;

fn handle_url<U: UiObject>(url: String) -> Result<HttpResponse, Error> {
    // parse url
    let parsed_url = ParsedUrl::new(url.to_string());

    // send a HTTP request and get a response
    let client = HttpClient::new();
    let response = match client.get(&parsed_url) {
        Ok(res) => {
            // redirect to Location
            if res.status_code() == 302 {
                let parsed_redirect_url = ParsedUrl::new(res.header("Location"));

                let redirect_client = HttpClient::new();
                let redirect_res = match redirect_client.get(&parsed_redirect_url) {
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

    Ok(response)
}

fn main() {
    // initialize the UI object
    let ui = Rc::new(RefCell::new(Tui::new()));

    // initialize the main browesr struct
    let mut browser = Browser::new(ui.clone());
    ui.borrow_mut().set_page(Rc::downgrade(&browser.page()));

    browser.start(handle_url::<Tui>);
}
