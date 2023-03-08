extern crate alloc;

use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use net::http::HttpClient;
use renderer::frame::frame::Frame;
use renderer::html::dom::*;
use renderer::js::ast::Program;
use renderer::layout::layout_object::*;
use renderer::ui::UiObject;
use url::ParsedUrl;

/// for debug
fn print_dom(node: &Option<Rc<RefCell<Node>>>, depth: usize) {
    match node {
        Some(n) => {
            print!("{}", "  ".repeat(depth));
            println!("{:?}", n.borrow().kind());
            print_dom(&n.borrow().first_child(), depth + 1);
            print_dom(&n.borrow().next_sibling(), depth);
        }
        None => return,
    }
}

/// for debug
fn print_render_object(node: &Option<Rc<RefCell<LayoutObject>>>, depth: usize) {
    match node {
        Some(n) => {
            print!("{}", "  ".repeat(depth));
            println!("{:?} {:?}", n.borrow().kind(), n.borrow().style);
            print_render_object(&n.borrow().first_child(), depth + 1);
            print_render_object(&n.borrow().next_sibling(), depth);
        }
        None => return,
    }
}

/// for debug
fn print_ast(program: &Program) {
    for node in program.body() {
        println!("{:?}", node);
    }
}

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
