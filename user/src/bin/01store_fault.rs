#![no_std]
#![no_main]

use log::info;

extern crate user;

#[unsafe(no_mangle)]
fn main() -> i32 {
    info!("Into Test store_fault, we will insert an invalid store operation...");
    unsafe {
        (0xb0000000 as *mut u8).write_volatile(0);
    }
    0
}