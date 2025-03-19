mod context;
mod info;
mod switch;

use context::{TaskContext, TaskControlBlock, TaskStatus};
use info::TimeInfo;
use lazy_static::*;
use log::{info, trace};
use switch::__switch;

use crate::{
    config::MAX_APP_NUM,
    loader::{get_num_app, init_app_cx},
    misc::terminate,
    sync::UPSafeCell,
};

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
    time_info: UPSafeCell<TimeInfo>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

// 定义全局变量
lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock::default(); MAX_APP_NUM];
        for i in 0..num_app {
            tasks[i].context = TaskContext::goto_restore(init_app_cx(i));
            tasks[i].status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
            time_info: unsafe { UPSafeCell::new(TimeInfo::new()) },
        }
    };
}

impl TaskManager {
    fn get_current_task_id(&self) -> usize {
        self.inner.exclusive_access().current_task
    }

    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.context as *const TaskContext;
        let mut __unused = TaskContext::default();
        drop(inner);

        // 记录时间
        let mut time_info = self.time_info.exclusive_access();
        time_info.record_start_time();
        drop(time_info);

        unsafe {
            __switch(&mut __unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn find_next_task(&self) -> Option<usize> {
        // get_next_app();
        let inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        ((current_task + 1)..(current_task + 1 + self.num_app))
            .map(|i| i % self.num_app)
            .find(|i| inner.tasks[*i].status == TaskStatus::Ready)
    }

    fn switch_to_next(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].context as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].context as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually

            trace_switch(current, next);

            // 记录时间
            let mut time_info = self.time_info.exclusive_access();
            time_info.record_start_time();
            drop(time_info);

            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            println!("All applications completed!");
            terminate();
        }
    }

    fn change_state_to_ready(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        inner.tasks[current_task].status = TaskStatus::Ready;
    }

    fn change_state_to_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        inner.tasks[current_task].status = TaskStatus::Exited;
    }
}
pub fn init() {
    TASK_MANAGER.run_first_task();
}

fn switch_to_next() {
    TASK_MANAGER.switch_to_next();
}

fn change_state_to_exited() {
    TASK_MANAGER.change_state_to_exited();
}

fn change_state_to_ready() {
    TASK_MANAGER.change_state_to_ready();
}

pub fn suspend_current_and_run_next() {
    change_state_to_ready();
    switch_to_next();
}
pub fn exit_current_and_run_next() {
    change_state_to_exited();

    // 记录时间
    let mut time_info  = TASK_MANAGER.time_info.exclusive_access();
    let current = TASK_MANAGER.get_current_task_id();
    time_info.record_end_time(current);
    
    // 打印时间信息
    let current = TASK_MANAGER.get_current_task_id();
    info!("Task {} run time: {} ms", current, time_info.get_run_time(current));
    drop(time_info);

    switch_to_next();
}
fn trace_switch(current: usize, next: usize) {
    if current != next {
        trace!("switch task from {} to {}", current,next);
    }
}