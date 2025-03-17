//use crate::batch::run_next_app;
use log::*;

use crate::batch::run_next_app;

pub fn sys_exit(xstate: i32) -> ! {
    info!("[kernel] Application exited with code {}", xstate);
    run_next_app()
}