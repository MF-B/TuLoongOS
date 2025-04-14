mod fs;
mod process;
mod time;
use crate::trap::context::TrapFrame;
use fs::{sys_close, sys_open, sys_read};
pub use fs::sys_write;
pub use process::sys_exit;
use process::{sys_exec, sys_fork, sys_getpid, sys_waitpid};
pub use process::sys_yield;
use time::sys_get_time;

const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_GETPID: usize = 172;


pub fn syscall(args: &mut TrapFrame,syscall_id: usize) -> isize {
    match syscall_id {
        SYSCALL_OPEN => sys_open(args.regs.a0 as *const u8, args.regs.a1 as u32),
        SYSCALL_CLOSE => sys_close(args.regs.a0),
        SYSCALL_WRITE => sys_write(args.regs.a0, args.regs.a1 as *const u8, args.regs.a2),
        SYSCALL_EXIT => sys_exit(args.regs.a0 as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_READ => sys_read(args.regs.a0, args.regs.a1 as *const u8, args.regs.a2),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args.regs.a0 as *const u8),
        SYSCALL_WAITPID => sys_waitpid(args.regs.a0 as isize, args.regs.a1 as *mut i32),
        SYSCALL_GETPID => sys_getpid(),
        _ => {
            panic!("Unsupported syscall_id: {}", syscall_id);
        },
    }
}