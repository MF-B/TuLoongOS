#![no_std]
#![no_main]

#[macro_use]
extern crate user;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("\nHello World\n");
    0
}
