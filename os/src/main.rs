#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

use core::arch::global_asm;
use config::print_machine_info;
use drivers::block::ahci_init;
use log::*;
use task::processor;

#[macro_use]
mod console;
mod misc;
mod lang_items;
mod logging;
mod logo;

mod syscall;
mod sync;
mod trap;
mod config;
mod task;
mod timer;
mod mm;
mod drivers;
pub mod fs;

extern crate alloc;
extern crate bitflags;

global_asm!(include_str!("entry.asm"));


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

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    unsafe extern "C" {
        safe fn stext(); // begin addr of text segment
        safe fn etext(); // end addr of text segment
        safe fn srodata(); // start addr of Read-Only data segment
        safe fn erodata(); // end addr of Read-Only data ssegment
        safe fn sdata(); // start addr of data segment
        safe fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        safe fn boot_stack_lower_bound(); // stack lower bound
        safe fn boot_stack_top(); // stack top
    }
    clear_bss();
    logo::print_logo();
    // 初始化日志系统
    logging::init();
    info!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize, etext as usize
    );
    info!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    info!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    info!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    print_machine_info();
    info!("协作式调度系统启动中...");
    mm::init();
    trap::init();
    ahci_init();
    // 启动第一个用户进程
    task::add_initproc();
    fs::list_apps();
    processor::run_tasks();

    // 关机
    misc::terminate();
}
