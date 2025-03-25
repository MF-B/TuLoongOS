#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

use core::arch::global_asm;
use config::print_machine_info;
use log::*;
use mm::{frame_allocator_test, heap_test, init_frame_allocator, init_heap, set_mmu};

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
mod loader;
mod config;
mod task;
mod timer;
mod mm;

extern crate alloc;
extern crate bitflags;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));


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
    set_mmu();
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
    init_heap();
    heap_test();
    init_frame_allocator();
    frame_allocator_test();
    info!("协作式调度系统启动中...");
    trap::init();
    loader::load_app();
    trap::enable_timer_interrupt();
    task::init();
    // 关机
    misc::terminate();
}
