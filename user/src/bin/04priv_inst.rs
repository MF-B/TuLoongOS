#![no_std]
#![no_main]

extern crate user;

use core::arch::asm;

use log::warn;

#[unsafe(no_mangle)]
fn main() -> i32 {
    warn!("Try to execute privileged instruction in U Mode");
    warn!("Kernel should kill this application!");
    unsafe {
        asm!("ertn");
    }
    0
}
