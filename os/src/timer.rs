use riscv::register::time;

use crate::sbi;

const CLOCK_FREQ: usize = 12500000;
const TICKS_PER_SEC: usize = 100;
const US_PER_SEC: usize = 1_000_000;

pub fn set_next_trigger(){
    sbi::set_timer(time::read64() as usize + (CLOCK_FREQ/TICKS_PER_SEC));
}

pub fn get_time_us() -> usize{
    time::read64() as usize / (CLOCK_FREQ / US_PER_SEC)
}