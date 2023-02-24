// dummy function
// TODO: remove this!
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        //$crate::io::_print($crate::format_args!($($arg)*));
    }};
}
