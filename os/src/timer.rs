use loongArch64::register::tcfg;
use loongArch64::time;

const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1000;

pub fn get_time() -> usize {
    time::Time::read()
}

pub fn set_next_trigger() {
    tcfg::set_init_val(get_time() + time::get_timer_freq() / TICKS_PER_SEC);
}

pub fn get_time_ms() -> usize {
    get_time() / (time::get_timer_freq() / MICRO_PER_SEC)
}