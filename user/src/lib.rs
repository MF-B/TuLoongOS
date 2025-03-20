#![no_std]
#![feature(linkage)]
#[macro_use]

pub mod console;
mod syscall;  // 确保有一个 syscall.rs 文件
mod lang_items;

pub use console::*;


#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();

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

fn clear_bss() {
    unsafe extern "C" {
        fn s_bss();
        fn e_bss();
    }
    (s_bss as usize..e_bss as usize).for_each(|addr| {
        unsafe { (addr as *mut u8).write_volatile(0); }
    });
}

use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize { sys_write(fd, buf) }
pub fn exit(exit_code: i32) -> isize { sys_exit(exit_code) }
pub fn yield_() -> isize { sys_yield() }
pub fn get_time() -> isize { sys_get_time()}