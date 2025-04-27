use alloc::{collections::vec_deque::VecDeque, sync::Arc};

use crate::task::{block_current_and_run_next, manager::wakeup_task, processor::current_task, suspend_current_and_run_next, task::TaskControlBlock};

use super::UPSafeCell;

pub trait Mutex: Sync + Send {
    fn lock(&self);
    fn unlock(&self);
}

pub struct MutexSpin {
    locked: UPSafeCell<bool>,  //locked是被UPSafeCell包裹的布尔全局变量
}

pub struct MutexBlocking {
    inner: UPSafeCell<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl MutexSpin {
    pub fn new() -> Self {
        Self {
            locked: unsafe { UPSafeCell::new(false) },
        }
    }
}
impl Mutex for MutexSpin {
    fn lock(&self) {
        loop {
            let mut locked = self.locked.exclusive_access();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                *locked = true;
                return;
            }
        }
    }

    fn unlock(&self) {
        *self.locked.exclusive_access() = false;
    }
}

impl MutexBlocking {
    pub fn new() -> Self {
        Self {
            inner: unsafe { UPSafeCell::new(MutexBlockingInner { locked: false, wait_queue: VecDeque::new() }) },
        }
    }
}
impl Mutex for MutexBlocking {
    fn lock(&self) {
        let mut inner = self.inner.exclusive_access();
        if inner.locked {
            // 如果锁被占用，则将当前任务加入等待队列
            inner.wait_queue.push_back(current_task().unwrap());
            drop(inner);
            block_current_and_run_next();
        }else {
            inner.locked = true;
        }
    }

    fn unlock(&self) {
        let mut inner = self.inner.exclusive_access();
        assert!(inner.locked);
        if let Some(task) = inner.wait_queue.pop_front() {
            wakeup_task(task);
        } else { 
            inner.locked = false;
        }
    }
}