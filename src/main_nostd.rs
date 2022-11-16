#![no_std]
#![no_main]
#![feature(start)]

//pub mod http;
//pub mod net;
//pub mod renderer;
//pub mod stdlib;
//pub mod url;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!();
}

#[no_mangle]
fn entry() -> isize {
    return -42;
}
