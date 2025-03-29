/// LoongArch QEMU虚拟机的串口地址
const UART_BASE_ADDR: usize = 0x1fe001e0;

/// 写入一个字节到UART
pub fn putc(c: u8) {
    unsafe {
        (UART_BASE_ADDR as *mut u8).write_volatile(c);
    }
}

const UART_DATA: usize = UART_BASE_ADDR;
const UART_LSR: usize = UART_BASE_ADDR + 5; // 线路状态寄存器
const UART_LSR_DR: u8 = 1; // 数据就绪位

/// 从UART读取一个字节
pub fn getc() -> usize {
    unsafe {
        // 检查是否有数据可读
        if (UART_LSR as *const u8).read_volatile() & UART_LSR_DR != 0 {
            // 有数据，读取并返回
            (UART_DATA as *const u8).read_volatile() as usize
        } else {
            // 无数据可读
            0
        }
    }
}
