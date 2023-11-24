#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use crate::alloc::string::ToString;
use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;
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

#[derive(Default)]
pub struct Allocator;

// TODO: implement allocator
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
        //malloc(layout.size() as u32) as *mut u8
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        //free(ptr as *mut c_void);
    }
}

#[alloc_error_handler]
fn my_allocator_error(_layout: Layout) -> ! {
    panic!("out of memory");
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

fn handle_url<U: UiObject>(url: String) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::new(
        "1.1".to_string(),
        200,
        "".to_string(),
        Vec::new(),
        "<h1>example</h1>".to_string(),
    ))
}

fn main() -> u64 {
    sys_print("**** Hello from an app!\n");

    // initialize the UI object
    let ui = Rc::new(RefCell::new(WasabiUI::new()));
    let page = Rc::new(RefCell::new(Page::new()));

    // initialize the main browesr struct
    let browser = Rc::new(RefCell::new(Browser::new(ui.clone(), page.clone())));

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
