

use lazy_static::*;

use context::{TaskContext, TaskControlBlock, TaskState};
use log::info;
use switch::__switch;

use crate::{config::MAX_APP_NUM, loader::get_num_app, sync::UPSafeCell};

mod context;
mod switch;

struct TaskManager {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

// 定义全局变量
lazy_static! {
    static ref TASK_MANAGER: UPSafeCell<TaskManager> = unsafe {
        UPSafeCell::new({
            // 设置返回值
            TaskManager {
                tasks: [TaskControlBlock::default(); MAX_APP_NUM],
                current_task: 0,
            }
        })
    };
}

pub fn get_next_app() -> usize {
    let num_app = get_num_app();
    let task_manager = TASK_MANAGER.exclusive_access();
    let current_task = task_manager.current_task;
    let next_task = (current_task + 1) % num_app;
    if task_manager.tasks[next_task].state != TaskState::Ready {
        for i in 0..num_app {
            if task_manager.tasks[i].state == TaskState::Ready {
                return i;
            }
        }
    } else {
        return next_task;
    }
    task_manager.current_task
}


pub fn switch_to_next() {
    let num_app = get_num_app();
    let mut task_manager = TASK_MANAGER.exclusive_access();
    let current_task = task_manager.current_task;
    task_manager.tasks[current_task].state = TaskState::Ready;
    let next_task = (current_task + 1) % num_app;
    if task_manager.tasks[next_task].state != TaskState::Ready {
        for i in 0..num_app {
            if task_manager.tasks[i].state == TaskState::Ready {
                task_manager.current_task = i;
                info!("switch to task {}", i);
                task_manager.tasks[i].state = TaskState::Running;
                break;
            }
        }
    } else {
        task_manager.current_task = next_task;
        task_manager.tasks[next_task].state = TaskState::Running;
    }

    let current_task_cx_ptr = &mut task_manager.tasks[current_task].context as *mut TaskContext;
    let next_task_cx_ptr = &mut task_manager.tasks[next_task].context as *mut TaskContext;

    drop(task_manager);

    unsafe {
        __switch(current_task_cx_ptr, next_task_cx_ptr);
    }
}


pub fn is_all_done() -> bool {
    let task_manager = TASK_MANAGER.exclusive_access();
    let num_app = get_num_app();
    for i in 0..num_app {
        if task_manager.tasks[i].state != TaskState::Blocked {
            return false
        }
    }
    true
}

