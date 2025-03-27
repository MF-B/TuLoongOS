mod context;
mod info;
mod switch;

use alloc::vec::Vec;
use context::{TaskContext, TaskControlBlock, TaskStatus};
use info::TimeInfo;
use lazy_static::*;
use log::info;
use loongArch64::register::pgdl;
use switch::__switch;

use crate::{
    config::PAGE_SIZE_BITS, loader::{get_app_data, get_app_trap_cx, get_num_app}, misc::terminate, sync::UPSafeCell
};

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
    time_info: UPSafeCell<TimeInfo>,
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

// 定义全局变量
lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(
                get_app_data(i),
                i,
            ));
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
    pub fn get_current_trap_cx(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let id = inner.tasks[inner.current_task].task_id;
        get_app_trap_cx(id)
    }

    fn get_current_task_id(&self) -> usize {
        self.inner.exclusive_access().current_task
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.context as *const TaskContext;
        let mut __unused = TaskContext::default();
        let pgd = task0.get_user_token() << PAGE_SIZE_BITS;
        pgdl::set_base(pgd);
        drop(inner);

        // 记录时间
        let mut time_info = self.time_info.exclusive_access();
        time_info.record_start_time();
        drop(time_info);

        info!("first task's pgd base is {:#x}", pgd);

        unsafe {
            __switch(&mut __unused as *mut TaskContext, next_task_cx_ptr, 1);
        }
        panic!("unreachable in run_first_task!");
    }

    fn find_next_task(&self) -> Option<usize> {
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
            let pgd = inner.tasks[next].get_user_token() << PAGE_SIZE_BITS;
            pgdl::set_base(pgd);
            drop(inner);
            // before this, we should drop local variables that must be dropped manually

            info_switch(current, next);

            // 记录时间
            let mut time_info = self.time_info.exclusive_access();
            time_info.record_start_time();
            drop(time_info);

            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr, next+1);
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
fn info_switch(current: usize, next: usize) {
    if current != next {
        info!("switch task from {} to {}", current,next);
    }
}

pub fn current_trap_cx() -> usize {
    TASK_MANAGER.get_current_trap_cx()
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}
