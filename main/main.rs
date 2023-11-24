#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;
use alloc::rc::Rc;
use core::cell::RefCell;
use noli::*;
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

fn main() -> u64 {
    sys_print("**** Hello from an app!\n");

    // initialize the UI object
    let ui = Rc::new(RefCell::new(WasabiUI::new()));

    sys_exit(42);
    0
}

entry_point!(main);
