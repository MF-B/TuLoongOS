#![no_std]
#![no_main]

extern crate user;

use core::arch::asm;

use user::println;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Try to execute privileged instruction in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        asm!("ertn");
    }
    0
}
