use crate::net::{FileDescriptor, SockAddr};
use alloc::alloc::{GlobalAlloc, Layout};
use alloc::string::String;
use core::option::Option;
use core::ptr::null_mut;

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        //$crate::io::_print($crate::format_args!($($arg)*));
    }};
}

pub fn socket(_domain: u32, _socket_type: u32, _protocol: u32) -> Option<FileDescriptor> {
    Some(FileDescriptor::new(42))
}

pub fn close(_fd: &FileDescriptor) -> i32 {
    42
}

pub fn sendto(
    _sockfd: &FileDescriptor,
    _buf: &mut String,
    _flags: u32,
    _dest_addr: &SockAddr,
) -> i64 {
    42
}

pub fn recvfrom(
    _sockfd: &FileDescriptor,
    _buf: &mut [u8],
    _flags: u32,
    _src_addr: &mut SockAddr,
) -> i64 {
    42
}

pub struct Window {}
impl Window {
    fn new() -> Self {
        Self {}
    }
}
pub fn create_window() -> Window {
    Window::new()
}

trait MutableAllocator {
    fn alloc(&mut self, layout: Layout) -> *mut u8;
    fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout);
}

const ALLOCATOR_BUF_SIZE: usize = 0x100000;
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
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
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
