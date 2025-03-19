use core::arch::global_asm;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

global_asm!(include_str!("syscall.S"));
unsafe extern "C" {
    pub fn __syscall(id: usize, args0: usize, args1: usize, args2: usize) -> isize;
}


fn syscall(id: usize, args: [usize; 3]) -> isize {
    unsafe {
        __syscall(id, args[0], args[1], args[2])
    }

}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}


pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}