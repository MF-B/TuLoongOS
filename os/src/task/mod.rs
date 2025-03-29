mod context;
mod switch;
mod pid;
pub mod task;
pub mod processor;
pub mod manager;

use core::arch::asm;

use alloc::sync::Arc;
use context::ProcessControlBlock;
use lazy_static::*;
use manager::add_process;
use processor::{schedule, take_current_task};
use task::{TaskContext, TaskStatus};
use crate::loader::get_app_data_by_name;

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = Arc::new(
        ProcessControlBlock::new(get_app_data_by_name("initproc").unwrap())
    );
}

pub fn add_initproc() {
    add_process(INITPROC.clone());
}

pub fn suspend_current_and_run_next() {
    // There must be an thread running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_context as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_process(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // 将当前进程从processor中移除
    let task = take_current_task().unwrap();
    let pid = task.getpid();

    // 标记为僵尸进程,并修改其exit_code码
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Zombie;
    task_inner.exit_code = exit_code;

    // 将当前进程的子进程的父进程设置为initproc
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in task_inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // 将当前进程的子进程从父进程中删除
    task_inner.children.clear();
    // 释放当前进程的数据页,但是保留页表结构
    task_inner.memory_set.recycle_data_pages();
    drop(task_inner);
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::default();
    unsafe {
        asm!("invtlb 0x4,{},$r0",in(reg) pid);
    }
    schedule(&mut _unused as *mut _);
}