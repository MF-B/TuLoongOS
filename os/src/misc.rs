use crate::println;

// GED 关机设备的具体物理地址：0x100E001C
const HALT_ADDR: *mut u8 = (0x100E001C) as *mut u8;

/// Shutdown the whole system, including all CPUs.
pub fn terminate() -> ! {
    println!("Shutting down...");
    unsafe { HALT_ADDR.write_volatile(0x34) };
    println!("It should shutdown!");
    loop {}
}