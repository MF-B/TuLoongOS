use alloc::sync::Arc;
use lazy_static::*;
use log::trace;
use loongArch64::register::{asid, pgdl};
use crate::{config::PAGE_SIZE_BITS, sync::UPSafeCell, trap::TrapFrame};

use super::{context::{TaskContext, TaskStatus}, manager::fetch_task, process::ProcessControlBlock, switch::__switch, task::TaskControlBlock};

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::default(),
        }
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe {
        UPSafeCell::new(Processor::new())
    };
}

impl Processor {
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    token
}

pub fn current_process() -> Arc<ProcessControlBlock> {
    current_task().unwrap().process.upgrade().unwrap()
}

pub fn current_trap_cx() -> &'static mut TrapFrame {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .kstack
        .get_mut::<TrapFrame>()
}

pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let pid = task.get_pid();
            let pgd = task.get_user_token() << PAGE_SIZE_BITS;
            let old_tid  = processor.current.as_ref().map(|t| t.get_tid()).unwrap_or(0);
            let tid = task.get_tid();
            
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);

            pgdl::set_base(pgd);
            asid::set_asid(pid);
            
            // stop exclusively accessing coming task TCB manually
            processor.current = Some(task);
            // stop exclusively accessing processor manually
            drop(processor);

            if old_tid!=tid {
                // if the task is not the same as the current one, we need to switch
                // to the new task
                trace!("switching from thread{} to thread{}",old_tid, tid);
            }
            //debug!("switching to task {}", pid);
            unsafe {
                __switch(
                    idle_task_cx_ptr,
                    next_task_cx_ptr,
                );
            }
        }
    }
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);

    unsafe {
        __switch(
            switched_task_cx_ptr,
            idle_task_cx_ptr,
        );
    }
}

