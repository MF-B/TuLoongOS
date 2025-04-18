use alloc::{collections::{btree_map::BTreeMap, vec_deque::VecDeque}, sync::Arc};
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
    pub static ref PID2TCB: UPSafeCell<BTreeMap<usize, Arc<ProcessControlBlock>>> =
        unsafe { UPSafeCell::new(BTreeMap::new()) };
}

pub fn add_process(task: Arc<ProcessControlBlock>) {
    PID2TCB
        .exclusive_access()
        .insert(task.getpid(), Arc::clone(&task));
    PROCESS_MANAGER.exclusive_access().add_process(task);
}

pub fn fetch_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESS_MANAGER.exclusive_access().fetch_process()
}
pub fn pid2task(pid: usize) -> Option<Arc<ProcessControlBlock>> {
    let map = PID2TCB.exclusive_access();
    map.get(&pid).map(Arc::clone)
}

pub fn remove_from_pid2task(pid: usize) {
    let mut map = PID2TCB.exclusive_access();
    if map.remove(&pid).is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}