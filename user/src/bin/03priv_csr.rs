#![no_std]
#![no_main]

extern crate user;

use loongArch64::register::{crmd, CpuMode};
use user::println;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    crmd::set_plv(CpuMode::Ring0);
    0
}
