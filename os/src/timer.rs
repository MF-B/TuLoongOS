use core::cmp::Ordering;

use alloc::sync::Arc;
use loongArch64::register::tcfg;
use loongArch64::time::{self};
use alloc::collections::BinaryHeap;
use lazy_static::lazy_static;
use crate::sync::UPSafeCell;
use crate::task::manager::wakeup_task;
use crate::task::task::TaskControlBlock;

const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1000;

pub fn get_time() -> usize {
    time::Time::read()
}

pub fn set_next_trigger() {
    tcfg::set_init_val(get_time() + time::get_timer_freq() / TICKS_PER_SEC);
}

pub fn get_time_ms() -> usize {
    get_time() / (time::get_timer_freq() / MICRO_PER_SEC)
}

pub struct TimerCondVar {
    pub expire_ms: usize,
    pub task: Arc<TaskControlBlock>,
}

impl PartialEq for TimerCondVar {
    fn eq(&self, other: &Self) -> bool {
        self.expire_ms == other.expire_ms
    }
}
impl Eq for TimerCondVar {}
impl PartialOrd for TimerCondVar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let a = -(self.expire_ms as isize);
        let b = -(other.expire_ms as isize);
        Some(a.cmp(&b))
    }
}
impl Ord for TimerCondVar {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}



lazy_static! {
    static ref TIMERS: UPSafeCell<BinaryHeap<TimerCondVar>> =
        unsafe { UPSafeCell::new(BinaryHeap::<TimerCondVar>::new()) };
}

pub fn add_timer(expire_ms: usize, task: Arc<TaskControlBlock>) {
    let mut timers = TIMERS.exclusive_access();
    timers.push(TimerCondVar { expire_ms, task });
}

pub fn remove_timer(task: Arc<TaskControlBlock>) {
    let mut timers = TIMERS.exclusive_access();
    let mut temp = BinaryHeap::<TimerCondVar>::new();
    for condvar in timers.drain() {
        if Arc::as_ptr(&task) != Arc::as_ptr(&condvar.task) {
            temp.push(condvar);
        }
    }
    timers.clear();
    timers.append(&mut temp);  
}

pub fn check_timer() {
    let current_ms = get_time_ms();
    let mut timers = TIMERS.exclusive_access();
    while let Some(timer) = timers.peek() {
        if timer.expire_ms <= current_ms {
            // 调用 wakeup_task 唤醒超时线程
            wakeup_task(Arc::clone(&timer.task));
            timers.pop();
        } else {
            break;
        }
    }
}

