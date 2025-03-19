#![no_std]
#![no_main]

use user::{get_time, println, yield_};

extern crate user;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Hello, world!");
    let current_time_us = get_time();
    let wait_for = current_time_us + 5000*1000;
    println!("Time: {}s", get_time() / 1000 / 1000);
    while get_time() < wait_for {
        yield_();
    }
    println!("Test sleep OK!");
    get_time();
    println!("Time: {}s", get_time() / 1000 / 1000);
    0
}
