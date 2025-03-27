mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;
pub mod memory_set;
pub use frame_allocator::*;
pub use address::*;
pub use page_table::*;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    frame_allocator_test();
    //set_mmu();
}