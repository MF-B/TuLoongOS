use crate::timer::get_time_us;

// #[repr(C)]
// #[derive(Debug,Default)]
// pub struct TimeVal {
//     pub sec: usize,
//     pub usec: usize,
// }

pub fn sys_get_time() -> isize {
    // unsafe { ts.as_mut().unwrap().sec = get_time_us() };
    get_time_us() as isize
}