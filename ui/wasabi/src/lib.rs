#![no_std]
#![cfg_attr(not(target_os = "linux"), no_main)]
use noli::prelude::*;

extern crate alloc;

pub mod app;
mod cursor;
