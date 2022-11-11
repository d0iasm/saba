use crate::stdlib::*;
use alloc::string::String;
use alloc::vec::Vec;
use core::result::Result;

pub const AF_INET: u32 = 2;

/// For TCP.
pub const SOCK_STREAM: u32 = 1;
/// For UDP.
pub const _SOCK_DGRAM: u32 = 2;

#[repr(C)]
#[derive(Debug)]
struct InAddr {
    /// IP address with big endian
    s_addr: u32,
}

impl InAddr {
    fn new(s_addr: u32) -> Self {
        Self { s_addr }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SockAddr {
    sin_family: u16,
    sin_port: u16,
    in_addr: InAddr,
}

impl SockAddr {
    pub fn new(sin_family: u16, sin_port: u16, s_addr: u32) -> Self {
        Self {
            sin_family,
            sin_port,
            in_addr: InAddr::new(s_addr),
        }
    }
}

#[derive(Debug)]
pub struct FileDescriptor {
    fd: i32,
}

impl FileDescriptor {
    pub fn new(fd: i32) -> Self {
        Self { fd }
    }

    pub fn number(&self) -> i32 {
        self.fd
    }
}

fn ip_to_int(ip: &str) -> u32 {
    let ip_blocks: Vec<&str> = ip.split('.').collect();
    if ip_blocks.len() != 4 {
        return 0;
    }

    (ip_blocks[3].parse::<u32>().unwrap() << 24)
        | (ip_blocks[2].parse::<u32>().unwrap() << 16)
        | (ip_blocks[1].parse::<u32>().unwrap())
        | (ip_blocks[0].parse::<u32>().unwrap())
}

fn inet_addr(host: &str) -> u32 {
    let v: Vec<&str> = host.splitn(2, ':').collect();
    let ip = if v.len() == 2 || v.len() == 1 {
        v[0]
    } else {
        panic!("invalid host name: {}", host);
    };
    ip_to_int(ip)
}

fn htons(port: u16) -> u16 {
    if cfg!(target_endian = "big") {
        port
    } else {
        port.swap_bytes()
    }
}

struct TcpStream {
    socket_fd: FileDescriptor,
    socket_addr: SockAddr,
}

impl TcpStream {
    pub fn connect(socket_addr: SockAddr) -> Result<TcpStream, String> {
        let socket_fd = match socket(AF_INET, SOCK_STREAM, 0) {
            Some(fd) => fd,
            None => return Err("can't create a socket file descriptor".to_string()),
        };

        Ok(TcpStream {
            socket_fd,
            socket_addr,
        })
    }

    pub fn write(&mut self, request: &mut String) -> Result<usize, String> {
        if sendto(&self.socket_fd, request, 0, &self.socket_addr) < 0 {
            return Err(format!("failed to send a request {}", request));
        }

        Ok(42)
    }

    pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize, String> {
        let mut buf = [0; 1000];
        let length = recvfrom(&self.socket_fd, &mut buf, 0, &mut self.socket_addr);
        if length < 0 {
            return Err("failed to receive a response".to_string());
        }

        Ok(length as usize)
    }

    pub fn shutdown(&self) -> Result<(), String> {
        close(&self.socket_fd);
        Ok(())
    }
}
