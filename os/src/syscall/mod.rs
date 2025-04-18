mod fs;
mod process;
mod time;
use crate::{task::signal::SignalAction, trap::context::TrapFrame};
use fs::{sys_close, sys_dup, sys_open, sys_pipe, sys_read};
pub use fs::sys_write;
pub use process::sys_exit;
use process::{sys_exec, sys_fork, sys_getpid, sys_kill, sys_sigaction, sys_sigprocmask, sys_sigreturn, sys_waitpid};
pub use process::sys_yield;
use time::sys_get_time;

const SYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
const SYSCALL_SIGACTION: usize = 134;
const SYSCALL_SIGPROCMASK: usize = 135;
const SYSCALL_SIGRETURN: usize = 139;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;


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
        SYSCALL_EXEC => sys_exec(args.regs.a0 as *const u8, args.regs.a1 as *const usize),
        SYSCALL_WAITPID => sys_waitpid(args.regs.a0 as isize, args.regs.a1 as *mut i32),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_PIPE => sys_pipe(args.regs.a0 as *mut usize),
        SYSCALL_DUP => sys_dup(args.regs.a0),
        SYSCALL_KILL => sys_kill(args.regs.a0, args.regs.a1 as i32),
        SYSCALL_SIGACTION => sys_sigaction(
            args.regs.a0 as i32, 
            args.regs.a1 as *const SignalAction, 
            args.regs.a2 as *mut SignalAction),
        SYSCALL_SIGPROCMASK => sys_sigprocmask(args.regs.a0 as u32),
        SYSCALL_SIGRETURN => sys_sigreturn(),
        _ => {
            panic!("Unsupported syscall_id: {}", syscall_id);
        },
    }
}