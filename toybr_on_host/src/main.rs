extern crate alloc;

use alloc::string::String;
use net::http::HttpClient;
use renderer::frame::frame::Frame;
use renderer::ui::UiObject;
use url::ParsedUrl;

fn handle_url<U: UiObject>(url: String) -> Result<Frame<U>, String> {
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

    let frame = Frame::new(url, response.body());

    Ok(frame)
}

fn main() {
    let _ = ui::app::start(handle_url);
}
