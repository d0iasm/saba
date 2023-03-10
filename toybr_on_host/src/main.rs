extern crate alloc;

use alloc::rc::Rc;
use alloc::string::String;
use browser::Browser;
use core::cell::RefCell;
use net::http::HttpClient;
use net::http::HttpResponse;
use renderer::ui::UiObject;
use ui::app::Tui;
use url::ParsedUrl;

fn handle_url<U: UiObject>(url: String) -> Result<HttpResponse, String> {
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
                    Err(e) => return Err(format!("{:?}", e)),
                };

                redirect_res
            } else {
                res
            }
        }
        Err(e) => return Err(format!("failed to get http response: {:?}", e)),
    };

    Ok(response)
}

fn main() {
    let ui = Tui::new();
    let mut browser = Browser::new(Rc::new(RefCell::new(ui)));
    //let page = Rc::downgrade(&browser.page());
    let page = browser.page();
    browser.ui().borrow_mut().set_page(page);

    browser.start(handle_url::<Tui>);
}
