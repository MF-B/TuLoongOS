/// LoongArch QEMU虚拟机的串口地址
const UART_BASE_ADDR: usize = 0x1fe001e0;

/// 写入一个字节到UART
pub fn putc(c: u8) {
    unsafe {
        (UART_BASE_ADDR as *mut u8).write_volatile(c);
    }
}