#![no_std]
#![no_main]

use user::yield_;

#[macro_use]
extern crate user;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Hello, world!");
    yield_();
    println!("Hello, world!");
    0
}
