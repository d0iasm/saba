extern crate alloc;

use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use net::http::HttpClient;
use net::http::HttpResponse;
use toybr_core::browser::Browser;
use toybr_core::common::error::Error;
use toybr_core::common::ui::UiObject;
use toybr_core::renderer::page::Page;
use toybr_core::url::ParsedUrl;
use ui::app::Tui;

fn handle_url<U: UiObject>(url: String) -> Result<HttpResponse, Error> {
    // parse url
    let parsed_url = ParsedUrl::new(url.to_string());

    // send a HTTP request and get a response
    let client = HttpClient::new();
    let response = match client.get(parsed_url.host, parsed_url.port, parsed_url.path) {
        Ok(res) => {
            // redirect to Location
            if res.status_code() == 302 {
                let parsed_redirect_url = ParsedUrl::new(res.header("Location"));

                let redirect_client = HttpClient::new();
                let redirect_res = match redirect_client.get(
                    parsed_redirect_url.host,
                    parsed_redirect_url.port,
                    parsed_redirect_url.path,
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

    Ok(response)
}

fn main() {
    // initialize the UI object
    let ui = Rc::new(RefCell::new(Tui::new()));
    let page = Rc::new(RefCell::new(Page::new()));

    // initialize the main browesr struct
    let browser = Rc::new(RefCell::new(Browser::new(ui.clone(), page.clone())));
    ui.borrow_mut().set_browser(Rc::downgrade(&browser));
    page.borrow_mut().set_browser(Rc::downgrade(&browser));

    match ui.borrow_mut().start(handle_url::<Tui>) {
        Ok(_) => {}
        Err(e) => {
            println!("browser fails to start {:?}", e);
        }
    };
}
