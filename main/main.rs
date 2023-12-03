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
use core::ptr;
use core::ptr::null_mut;
use noli::*;
use toybr_core::browser::Browser;
use toybr_core::error::Error;
use toybr_core::http::HttpResponse;
use toybr_core::renderer::page::Page;
use toybr_core::ui::UiObject;
use ui_wasabi::WasabiUI;

trait MutableAllocator {
    fn alloc(&mut self, layout: Layout) -> *mut u8;
    fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout);
}

const ALLOCATOR_BUF_SIZE: usize = 0x10000;
pub struct WaterMarkAllocator {
    buf: [u8; ALLOCATOR_BUF_SIZE],
    used_bytes: usize,
}

pub struct GlobalAllocatorWrapper {
    allocator: WaterMarkAllocator,
}

#[global_allocator]
static mut ALLOCATOR: GlobalAllocatorWrapper = GlobalAllocatorWrapper {
    allocator: WaterMarkAllocator {
        buf: [0; ALLOCATOR_BUF_SIZE],
        used_bytes: 0,
    },
};

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

impl MutableAllocator for WaterMarkAllocator {
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if self.used_bytes > ALLOCATOR_BUF_SIZE {
            return null_mut();
        }
        self.used_bytes = (self.used_bytes + layout.align() - 1) / layout.align() * layout.align();
        self.used_bytes += layout.size();
        if self.used_bytes > ALLOCATOR_BUF_SIZE {
            return null_mut();
        }
        unsafe { self.buf.as_mut_ptr().add(self.used_bytes - layout.size()) }
    }
    fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {}
}
unsafe impl GlobalAlloc for GlobalAllocatorWrapper {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATOR.allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATOR.allocator.dealloc(ptr, layout);
    }
}

/*
#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

/// The implementation comes from https://os.phil-opp.com/allocator-designs/
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Creates a new empty bump allocator.
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get a mutable reference

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut() // out of memory
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // get a mutable reference

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

#[alloc_error_handler]
fn my_allocator_error(_layout: Layout) -> ! {
    panic!("out of memory");
}
*/

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
