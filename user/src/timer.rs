#[repr(C)]
#[derive(Debug,Default)]
pub struct TimeVal {
    pub sec: isize,
    pub usec: usize,
}