use core::arch::global_asm;

use crate::SignalAction;

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
const SYSCALL_THREAD_CREATE: usize = 1000;
const SYSCALL_GETTID: usize = 1001;
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


global_asm!(include_str!("syscall.S"));
unsafe extern "C" {
    pub fn __syscall(id: usize, args0: usize, args1: usize, args2: usize) -> isize;
}


fn syscall(id: usize, args: [usize; 3]) -> isize {
    unsafe {
        __syscall(id, args[0], args[1], args[2])
    }

}

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0])
}


pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}


pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}

pub fn sys_exec(path: &str, args: &[*const u8]) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, args.as_ptr() as usize, 0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_mut_ptr() as usize, buffer.len()])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}
pub fn sys_pipe(pipe: &mut [usize]) -> isize {
    syscall(SYSCALL_PIPE, [pipe.as_mut_ptr() as usize, 0, 0])
}
pub fn sys_dup(fd: usize) -> isize {
    syscall(SYSCALL_DUP, [fd, 0, 0])
}

pub fn sys_kill(pid: usize, signal: i32) -> isize {
    syscall(SYSCALL_KILL, [pid, signal as usize, 0])
}
pub fn sys_sigaction(
    signum: i32,
    action: *const SignalAction,
    old_action: *mut SignalAction,
) -> isize {
    syscall(
        SYSCALL_SIGACTION,
        [signum as usize, action as usize, old_action as usize],
    )
    /*
    syscall(
        SYSCALL_SIGACTION,
        [
            signum as usize,
            action.map_or(0, |r| r as *const _ as usize),
            old_action.map_or(0, |r| r as *mut _ as usize),
        ],
    )
    */
}

pub fn sys_sigprocmask(mask: u32) -> isize {
    syscall(SYSCALL_SIGPROCMASK, [mask as usize, 0, 0])
}

pub fn sys_sigreturn() -> isize {
    syscall(SYSCALL_SIGRETURN, [0, 0, 0])
}

/// 功能：当前进程创建一个新的线程
/// 参数：entry 表示线程的入口函数地址
/// 参数：arg：表示线程的一个参数
pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    syscall(SYSCALL_THREAD_CREATE, [entry, arg, 0])
}
/// 参数：tid表示线程id
/// 返回值：如果线程不存在，返回-1；如果线程还没退出，返回-2；其他情况下，返回结束线程的退出码
pub fn sys_gettid() -> isize {
    syscall(SYSCALL_GETTID, [0, 0, 0])
}

pub fn sys_waittid(tid: usize) -> isize {
    syscall(SYSCALL_WAITTID, [tid, 0, 0])
}

pub fn sys_mutex_create(blocking: bool) -> isize {
    syscall(SYSCALL_MUTEX_CREATE, [blocking as usize, 0, 0])
}

pub fn sys_mutex_lock(id: usize) -> isize {
    syscall(SYSCALL_MUTEX_LOCK, [id, 0, 0])
}

pub fn sys_mutex_unlock(id: usize) -> isize {
    syscall(SYSCALL_MUTEX_UNLOCK, [id, 0, 0])
}

/// 功能：为当前进程新增一个信号量。
/// 参数：res_count 表示该信号量的初始资源可用数量，即 N ，为一个非负整数。
/// 返回值：假定该操作必定成功，返回创建的信号量的 ID 。
/// syscall ID : 1020
pub fn sys_semaphore_create(res_count: usize) -> isize {
    syscall(SYSCALL_SEMAPHORE_CREATE, [res_count, 0, 0])
}
/// 功能：对当前进程内的指定信号量进行 V 操作。
/// 参数：sem_id 表示要进行 V 操作的信号量的 ID 。
/// 返回值：假定该操作必定成功，返回 0 。
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    syscall(SYSCALL_SEMAPHORE_UP, [sem_id, 0, 0])
}
/// 功能：对当前进程内的指定信号量进行 P 操作。
/// 参数：sem_id 表示要进行 P 操作的信号量的 ID 。
/// 返回值：假定该操作必定成功，返回 0 。
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    syscall(SYSCALL_SEMAPHORE_DOWN, [sem_id, 0, 0])
}

/// 功能：为当前进程新增一个条件变量。
/// 返回值：假定该操作必定成功，返回创建的条件变量的 ID 。
/// syscall ID : 1030
pub fn sys_condvar_create() -> isize {
    syscall(SYSCALL_CONDVAR_CREATE, [0, 0, 0])
}

/// 功能：对当前进程的指定条件变量进行 signal 操作，即
/// 唤醒一个在该条件变量上阻塞的线程（如果存在）。
/// 参数：condvar_id 表示要操作的条件变量的 ID 。
/// 返回值：假定该操作必定成功，返回 0 。
/// syscall ID : 1031
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    syscall(SYSCALL_CONDVAR_SIGNAL, [condvar_id, 0, 0])
}

/// 功能：对当前进程的指定条件变量进行 wait 操作，分为多个阶段：
/// 1. 释放当前线程持有的一把互斥锁；
/// 2. 阻塞当前线程并将其加入指定条件变量的阻塞队列；
/// 3. 直到当前线程被其他线程通过 signal 操作唤醒；
/// 4. 重新获取当前线程之前持有的锁。
/// 参数：mutex_id 表示当前线程持有的互斥锁的 ID ，而
/// condvar_id 表示要操作的条件变量的 ID 。
/// 返回值：假定该操作必定成功，返回 0 。
/// syscall ID : 1032
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    syscall(SYSCALL_CONDVAR_WAIT, [condvar_id, mutex_id, 0])
}