use alloc::sync::Arc;
use log::*;

use crate::{loader::get_app_data_by_name, mm::{translated_refmut, translated_str}, task::{exit_current_and_run_next, manager::add_process, processor::{current_process, current_user_token}, suspend_current_and_run_next},trap::TrapFrame};

pub fn sys_exit(exit_code: i32) -> ! {
    let pid = current_process().unwrap().getpid();
    info!("Process: {} exited with code {}", pid, exit_code);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    //trace!("Application yield");
    suspend_current_and_run_next();
    0
}

pub fn sys_fork() -> isize {
    let current_task = current_process().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let mut trap_cx: TrapFrame = *new_task.kernel_stack.get_mut();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.regs.a0 = 0;  //x[10] is a0 reg
    // add new task to scheduler
    add_process(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        debug!("Application exec {}", path);
        let task = current_process().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_process().unwrap();
    // find a child process

    // ---- access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    if inner.children
        .iter()
        .find(|p| {pid == -1 || pid as usize == p.getpid()})
        .is_none() {
        return -1;
        // ---- stop exclusively accessing current PCB
    }
    let pair = inner.children
        .iter()
        .enumerate()
        .find(|(_, p)| {
            // ++++ temporarily access child PCB exclusively
            p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
            // ++++ stop exclusively accessing child PCB
        });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child TCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ stop exclusively accessing child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- stop exclusively accessing current PCB automatically
}

pub fn sys_getpid() -> isize {
    current_process().unwrap().getpid() as isize
}
