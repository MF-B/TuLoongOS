use alloc::sync::Arc;

use crate::{sync::{mutex::{Mutex, MutexBlocking, MutexSpin}, semaphore::Semaphore}, task::{block_current_and_run_next, processor::{current_process, current_task}}, timer::{add_timer, get_time_ms}};


pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}

pub fn sys_mutex_create(blocking: bool) -> isize {
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() as isize - 1
    }
}

pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.lock();
    0
}
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}

pub fn sys_semaphore_create(res_count: usize) -> isize {
    let process = current_process();
    let semaphore = Some(Arc::new(Semaphore::new(res_count)));
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = semaphore;
        id as isize
    } else {
        process_inner.semaphore_list.push(semaphore);
        process_inner.semaphore_list.len() as isize - 1
    }
}
pub fn sys_semaphore_up(semaphore_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let semaphore = Arc::clone(process_inner.semaphore_list[semaphore_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    semaphore.up();
    0
}
pub fn sys_semaphore_down(semaphore_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let semaphore = Arc::clone(process_inner.semaphore_list[semaphore_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    semaphore.down();
    0
}