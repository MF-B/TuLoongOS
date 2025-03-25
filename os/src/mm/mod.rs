mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;
pub mod memory_set;
use bit_field::BitField;
pub use heap_allocator::*;
use loongArch64::register::{ecfg::LineBasedInterrupt, *};
pub use frame_allocator::*;

pub fn set_mmu() {

}



pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    set_mmu();
}