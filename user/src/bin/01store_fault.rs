#![no_std]
#![no_main]

use user::println;

extern crate user;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Into Test store_fault, we will insert invalid store operations to a range...");
    
    // 定义起始地址和范围大小
    let start_addr: usize = 0x0;
    let range_size: usize = 8; // 设置要写入的字节数
    
    println!("Writing zeros to memory range: 0x{:x} - 0x{:x}", start_addr, start_addr + range_size - 1);
    
    unsafe {
        // 循环将一个地址范围内的每个字节都设置为0
        for offset in 0..range_size {
            ((start_addr + offset) as *mut u8).write_volatile(0);
        }
    }
    
    println!("If you see this, memory write didn't cause fault as expected");
    0
}