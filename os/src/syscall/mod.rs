mod fs;
mod process;
use crate::trap::context::TrapFrame;
pub use fs::sys_write;
pub use process::sys_exit;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

pub fn syscall(args: &mut TrapFrame,syscall_id: usize) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args.regs.a0, args.regs.a1 as *const u8, args.regs.a2),
        SYSCALL_EXIT => sys_exit(args.regs.a0 as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}