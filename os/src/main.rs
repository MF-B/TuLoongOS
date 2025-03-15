#![no_std]
#![no_main]
use core::arch::global_asm;
use log::*;
#[macro_use]
mod console;
mod lang_items;
mod uart;
mod misc;
mod logging;


global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();

    // 初始化日志系统
    logging::init();
    println!("Level:{}", log::max_level());

    // 日志测试
    error!("Hello, Navi!");
    warn!("Hello, Lain!");
    info!("Hello, MFYX!");
    debug!("Hello, TuloongOS!");
    trace!("Hello, 你是谁?!");

    // 进行一些操作后关机
    info!("Performing system shutdown...");
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