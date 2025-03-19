use crate::{config::MAX_APP_NUM, loader::get_num_app, timer::get_time_us};

pub struct TimeInfo {
    start_iter: usize,
    start_time: [usize;MAX_APP_NUM],
    end_time: [usize;MAX_APP_NUM],
}
impl TimeInfo{
    pub fn new() -> Self {
        Self {
            start_iter: 0,
            start_time: [0;MAX_APP_NUM],
            end_time: [0;MAX_APP_NUM],
        }
    }
    pub fn record_start_time(&mut self) {
        let num_app = get_num_app();
        if self.start_iter < num_app {
            self.start_time[self.start_iter] = get_time_us();
            self.start_iter += 1;
        }
    }
    pub fn record_end_time(&mut self, app_id:usize) {
        //let num_app = get_num_app();
        self.end_time[app_id] = get_time_us();
    }

    pub fn get_run_time(&self,app_id: usize) -> usize {
        let start_time = self.get_start_time(app_id);
        let end_time = self.get_end_time(app_id);
        (end_time - start_time) / 1000
    }

    pub fn get_start_time(&self, app_id: usize) -> usize {
        self.start_time[app_id]
    }

    pub fn get_end_time(&self, app_id: usize) -> usize {
        self.end_time[app_id]
    }
}