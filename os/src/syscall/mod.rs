mod fs;
mod process;
mod time;
mod thread;
mod sync;
use crate::trap::context::TrapFrame;
use fs::{sys_close, sys_dup, sys_open, sys_pipe, sys_read};
pub use fs::sys_write;
pub use process::sys_exit;
use process::{sys_exec, sys_fork, sys_getpid, sys_kill, sys_waitpid};
pub use process::sys_yield;
use sync::{sys_condvar_create, sys_condvar_signal, sys_condvar_wait, sys_mutex_create, sys_mutex_lock, sys_mutex_unlock, sys_semaphore_create, sys_semaphore_down, sys_semaphore_up, sys_sleep};
use thread::{sys_thread_create, sys_waittid};
use time::sys_get_time;

const SYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_SLEEP: usize = 101;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_THREAD_CREATE: usize = 1000;
const SYSCALL_WAITTID: usize = 1002;
const SYSCALL_MUTEX_CREATE: usize = 1010;
const SYSCALL_MUTEX_LOCK: usize = 1011;
const SYSCALL_MUTEX_UNLOCK: usize = 1012;
const SYSCALL_SEMAPHORE_CREATE: usize = 1020;
const SYSCALL_SEMAPHORE_UP: usize = 1021;
const SYSCALL_SEMAPHORE_DOWN: usize = 1022;
const SYSCALL_CONDVAR_CREATE: usize = 1030;
const SYSCALL_CONDVAR_SIGNAL: usize = 1031;
const SYSCALL_CONDVAR_WAIT: usize = 1032;


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
        SYSCALL_THREAD_CREATE => sys_thread_create(args.regs.a0,args.regs.a1),
        SYSCALL_WAITTID => sys_waittid(args.regs.a0),
        SYSCALL_SLEEP => sys_sleep(args.regs.a0),
        SYSCALL_MUTEX_CREATE => sys_mutex_create(args.regs.a0 != 0),
        SYSCALL_MUTEX_LOCK => sys_mutex_lock(args.regs.a0 as usize),
        SYSCALL_MUTEX_UNLOCK => sys_mutex_unlock(args.regs.a0 as usize),
        SYSCALL_SEMAPHORE_CREATE => sys_semaphore_create(args.regs.a0),
        SYSCALL_SEMAPHORE_UP => sys_semaphore_up(args.regs.a0),
        SYSCALL_SEMAPHORE_DOWN => sys_semaphore_down(args.regs.a0),
        SYSCALL_CONDVAR_CREATE => sys_condvar_create(),
        SYSCALL_CONDVAR_SIGNAL => sys_condvar_signal(args.regs.a0),
        SYSCALL_CONDVAR_WAIT => sys_condvar_wait(args.regs.a0, args.regs.a1),
        _ => {
            panic!("Unsupported syscall_id: {}", syscall_id);
        },
    }
}