use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use lazy_static::*;
use log::debug;
use crate::sync::UPSafeCell;

use super::context::ProcessControlBlock;

pub struct ProcessManager {
    ready_queue: VecDeque<Arc<ProcessControlBlock>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        debug!("ProcessManager init");
        ProcessManager {
            ready_queue: VecDeque::new(),
        }
    }

    pub fn add_process(&mut self, task: Arc<ProcessControlBlock>) {
        //debug!("Add process pid={}", task.pid.0);
        self.ready_queue.push_back(task);
    }

    pub fn fetch_process(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref PROCESS_MANAGER: UPSafeCell<ProcessManager> =
        unsafe { UPSafeCell::new(ProcessManager::new()) };
}

pub fn add_process(task: Arc<ProcessControlBlock>) {
    PROCESS_MANAGER.exclusive_access().add_process(task);
}

pub fn fetch_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESS_MANAGER.exclusive_access().fetch_process()
}
