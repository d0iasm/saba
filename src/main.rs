#![no_std]
#![no_main]

use core::panic::PanicInfo;

macro_rules! entry_point {
    // c.f. https://docs.rs/bootloader/0.6.4/bootloader/macro.entry_point.html
    ($path:path) => {
        #[no_mangle]
        pub unsafe extern "C" fn entry() -> i64 {
            // validate the signature of the program entry point
            let f: fn() -> i64 = $path;
            f()
        }
    };
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!();
}

fn main() -> i64 {
    return -42;
}

entry_point!(main);
