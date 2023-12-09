#![no_std]
#![no_main]

extern crate alloc;

use crate::alloc::string::ToString;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use noli::*;
use toybr_core::browser::Browser;
use toybr_core::error::Error;
use toybr_core::http::HttpResponse;
use toybr_core::renderer::page::Page;
use toybr_core::ui::UiObject;
use ui_wasabi::WasabiUI;

fn handle_url<U: UiObject>(url: String) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::new(
        "1.1".to_string(),
        200,
        "".to_string(),
        Vec::new(),
        r#"
<html>
<head>
  <style type="text/css">
    div {
      width: 600;
      margin: 200;
      background-color: white;
    }
  </style>
</head>
<body>
<div>
  <h1>Example Domain</h1>
  <p>This domain is for use in illustrative examples in documents. You may use this
  domain in literature without prior coordination or asking for permission.</p>
  <p><a href="http://localhost:8000/test2.html">More information...</a></p>
</div>
</body>
</html>"#
            .to_string(),
    ))
}

fn main() -> u64 {
    sys_print("**** Hello from an app!\n");

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
            sys_exit(1);
        }
    };
    0
}

entry_point!(main);
