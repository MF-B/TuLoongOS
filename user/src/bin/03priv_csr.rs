#![no_std]
#![no_main]

extern crate user;

use log::warn;
use loongArch64::register::{crmd, CpuMode};

#[unsafe(no_mangle)]
fn main() -> i32 {
    warn!("Try to access privileged CSR in U Mode");
    warn!("Kernel should kill this application!");
    crmd::set_plv(CpuMode::Ring0);
    0
}
