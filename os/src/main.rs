#![no_std]
#![no_main]
use core::arch::global_asm;
use log::*;
#[macro_use]
mod console;
mod uart;
mod misc;
mod lang_items;
mod logging;
mod logo;

mod syscall;
mod sync;
mod trap;
pub mod batch;
mod loader;
mod config;
mod task;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));


#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    logo::print_logo();
    // 初始化日志系统
    logging::init();
    info!("批处理系统启动中...");

    trap::init();

    batch::init();
    
    batch::run_next_app();

    // 关机
    //misc::terminate();
}

fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}