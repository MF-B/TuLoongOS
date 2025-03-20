//use crate::batch::run_next_app;
use log::*;

use crate::task::exit_current_and_run_next;
use crate::task::suspend_current_and_run_next;

pub fn sys_exit(xstate: i32) -> ! {
    info!("Application exited with code {}", xstate);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    //trace!("Application yield");
    suspend_current_and_run_next();
    0
}