#![no_std]
#![no_main]

use log::info;
use user::yield_;

extern crate user;

#[unsafe(no_mangle)]
fn main() -> i32 {
    info!("Hello, world!");
    yield_();
    info!("Hello, world!");
    0
}
