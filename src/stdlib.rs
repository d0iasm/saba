use crate::net::{FileDescriptor, SockAddr};
use core::option::Option;

macro_rules! println {
    ($($arg:tt)*) => {{
        $crate::io::_print($crate::format_args!($($arg)*));
    }};
}

pub fn socket(domain: u32, socket_type: u32, protocol: u32) -> Option<FileDescriptor> {
    Some(FileDescriptor::new(42))
}

pub fn close(fd: &FileDescriptor) -> i32 {
    42
}

pub fn sendto(sockfd: &FileDescriptor, buf: &mut String, flags: u32, dest_addr: &SockAddr) -> i64 {
    42
}

pub fn recvfrom(
    sockfd: &FileDescriptor,
    buf: &mut [u8],
    flags: u32,
    src_addr: &mut SockAddr,
) -> i64 {
    42
}
