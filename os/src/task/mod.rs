
use lazy_static::*;

use context::{TaskContext, TaskControlBlock, TaskState};
use log::{info, trace};
use switch::__switch;

use crate::{
    config::MAX_APP_NUM, loader::{get_num_app, init_app_cx}, misc::terminate, sync::UPSafeCell
};

mod context;
mod switch;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
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
            tasks[i].state = TaskState::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.state = TaskState::Running;
        let next_task_cx_ptr = &task0.context as *const TaskContext;
        let mut __unused = TaskContext::default();
        drop(inner);

        unsafe {
            __switch(&mut __unused as *mut TaskContext, next_task_cx_ptr);
        }
        //panic!("unreachable in run_first_task!");
    }
}
pub fn init() {
    TASK_MANAGER.run_first_task();
}

pub fn suspend_current_and_run_next() {
    change_state_to_ready();
    switch_to_next();
}
pub fn exit_current_and_run_next() {
    change_state_to_exited();
    switch_to_next();
}


fn change_state_to_ready() {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current_task = inner.current_task;
    inner.tasks[current_task].state = TaskState::Ready;
    drop(inner);
}
fn change_state_to_exited() {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current_task = inner.current_task;
    inner.tasks[current_task].state = TaskState::Exited;
    drop(inner);
}

fn find_next_task() -> usize {
    // get_next_app();
    let inner = TASK_MANAGER.inner.exclusive_access();
    let num_app = TASK_MANAGER.num_app;
    let current_task = inner.current_task;
    let next_task = ((current_task + 1)..(current_task + 1 + num_app))
        .map(|i| i % num_app)
        .find(|i| inner.tasks[*i].state == TaskState::Ready)
        .unwrap_or(current_task);
    drop(inner);
    next_task
}

fn switch_to_next(){
    let next = find_next_task();
    let mut inner = TASK_MANAGER.inner.exclusive_access();

    let current_task = inner.current_task;
    if current_task == next && inner.tasks[current_task].state == TaskState::Exited {
        info!("All Applications are done!");
        terminate();
    }
    trace!("switch from {} to {}", current_task, next);
    let current_task_cx_ptr = &mut inner.tasks[current_task].context as *mut TaskContext;
    let next_task_cx_ptr = &inner.tasks[next].context as *const TaskContext;
    inner.current_task = next;
    inner.tasks[next].state = TaskState::Running;
    drop(inner);
    unsafe { __switch(current_task_cx_ptr, next_task_cx_ptr) };
}



