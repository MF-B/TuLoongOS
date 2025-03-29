use alloc::sync::Arc;
use lazy_static::*;
use log::debug;
use loongArch64::register::{asid, pgdl};
use crate::{config::PAGE_SIZE_BITS, sync::UPSafeCell};

use super::{context::ProcessControlBlock, manager::fetch_process, switch::__switch, task::{TaskContext, TaskStatus}};

pub struct Processor {
    current: Option<Arc<ProcessControlBlock>>,
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
        debug!("Processor init");
        UPSafeCell::new(Processor::new())
    };
}

impl Processor {
    pub fn take_current(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<ProcessControlBlock>> {
        self.current.as_ref().map(|task| Arc::clone(task))
    }
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
}

pub fn take_current_task() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_task() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}

pub fn current_trap_cx() -> usize {
    let task = current_task().unwrap();
    task.get_trap_cx()
}

pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(process) = fetch_process() {
            
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = process.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_context as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;

            let pid = process.getpid();
            let pgd = task_inner.get_user_token() << PAGE_SIZE_BITS;
            pgdl::set_base(pgd);
            asid::set_asid(pid);
            
            // stop exclusively accessing coming task TCB manually
            drop(task_inner);
            processor.current = Some(process);
            // stop exclusively accessing processor manually
            drop(processor);

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
            idle_task_cx_ptr
        );
    }
}

