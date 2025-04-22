use alloc::{string::String, sync::Arc, vec::Vec};
use log::{debug, info, trace};

use crate::{fs::inode::{open_file, OpenFlags}, mm::{translated_ref, translated_refmut, translated_str}, task::{exit_current_and_run_next, manager::pid2process, processor::{current_process, current_task, current_user_token}, signal::SignalFlags, suspend_current_and_run_next}, trap::TrapFrame};

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    //trace!("Application yield");
    suspend_current_and_run_next();
    0
}

pub fn sys_fork() -> isize {
    trace!("sys_fork");
    let current_process = current_process();
    let new_process = current_process.fork();
    let new_pid = new_process.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let new_process_inner = new_process.inner_exclusive_access();
    let task = new_process_inner.tasks[0].as_ref().unwrap();
    let mut trap_cx: TrapFrame = *task.inner_exclusive_access().kstack.get_mut::<TrapFrame>();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.regs.a0 = 0;  //x[10] is a0 reg
    new_pid as isize
}

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    trace!("sys_exec");
    let token = current_user_token();
    let path = translated_str(token, path);

    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe { args = args.add(1); }
    }

    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let process = current_process();
        let argc = args_vec.len();
        process.exec(all_data.as_slice(), args_vec);
        argc as isize
    } else {
        trace!("exec failed");
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let process = current_process();
    // find a child process

    // ---- access current TCB exclusively
    let mut inner = process.inner_exclusive_access();
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
    current_task().unwrap().get_pid() as isize
}

pub fn sys_kill(pid: usize, signum: i32) -> isize {
    if let Some(task) = pid2process(pid) {
        info!("sys_kill: pid {}, signum {}", pid, signum);	
        if let Some(flag) = SignalFlags::from_bits(1 << signum) {
            // insert the signal if legal
            let mut task_ref = task.inner_exclusive_access();
            if task_ref.signals.contains(flag) {
                return -1;
            }
            task_ref.signals.insert(flag);
            0
        } else {
            -1
        }
    } else {
        info!("no task");	
        -1
    }
}