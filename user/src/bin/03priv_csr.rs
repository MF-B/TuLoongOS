#![no_std]
#![no_main]

#[macro_use]
extern crate user;

use loongArch64::register::{crmd, CpuMode};

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    crmd::set_plv(CpuMode::Ring0);
    0
}
