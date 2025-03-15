#![no_std]
#![no_main]
use core::arch::global_asm;

#[macro_use]
mod uart;
mod lang_items;
mod misc;
pub mod console;


global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    // 使用uart模块输出信息
    print!("print!\n");
    println!("Hello, {}LoongOS!",111);
    println!("Kernel initialized successfully!");

    println!("Debug!\n");
    print!("print!\n");

    // 进行一些操作后关机
    println!("Performing system shutdown...");
    misc::terminate();
}

fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}