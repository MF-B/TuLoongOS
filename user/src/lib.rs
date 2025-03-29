#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod syscall;  // 确保有一个 syscall.rs 文件
mod lang_items;
use core::ptr::addr_of_mut;

use buddy_system_allocator::LockedHeap;
pub use console::*;

const USER_HEAP_SIZE: usize = 16384;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    
    unsafe {
        //let heap_space = &raw mut HEAP_SPACE as usize;
        HEAP.lock()
            .init(addr_of_mut!(HEAP_SPACE) as usize, USER_HEAP_SIZE);
    }

    // unsafe extern "C" {
    //     safe fn s_text(); // begin addr of text segment
    //     safe fn e_text(); // end addr of text segment
    //     safe fn s_rodata(); // start addr of Read-Only data segment
    //     safe fn e_rodata(); // end addr of Read-Only data ssegment
    //     safe fn s_data(); // start addr of data segment
    //     safe fn e_data(); // end addr of data segment
    //     fn s_bss(); // start addr of BSS segment
    //     fn e_bss(); // end addr of BSS segment
    // }
    // println!(
    //     "[ user ] .text [{:#x}, {:#x})",
    //     s_text as usize, e_text as usize
    // );
    // println!(
    //     "[ user ] .rodata [{:#x}, {:#x})",
    //     s_rodata as usize, e_rodata as usize
    // );
    // println!(
    //     "[ user ] .data [{:#x}, {:#x})",
    //     s_data as usize, e_data as usize
    // );
    // println!("[ user ] .bss [{:#x}, {:#x})", s_bss as usize, e_bss as usize);

    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 {
    panic!("Cannot find main!");
}

use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize { sys_write(fd, buf) }
pub fn read(fd: usize, buf: &mut [u8]) -> isize { sys_read(fd, buf) }
pub fn exit(exit_code: i32) -> isize { sys_exit(exit_code) }
pub fn yield_() -> isize { sys_yield() }
pub fn get_time() -> isize { sys_get_time()}
pub fn fork() -> isize { sys_fork() }
pub fn exec(path: &str) -> isize { sys_exec(path) }
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => { yield_(); }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => { yield_(); }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}