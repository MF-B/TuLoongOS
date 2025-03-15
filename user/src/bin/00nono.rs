#![no_std]
#![no_main]

extern crate user;
// 定义程序入口点
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // 程序逻辑
    loop {}
}