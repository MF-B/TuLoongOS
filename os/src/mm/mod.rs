mod heap_allocator;
pub use heap_allocator::*;
use loongArch64::register::{crmd, dmw0, MemoryAccessType};

pub fn set_mmu() {
    // 为内核设置直接映射地址翻译模式
    dmw0::set_vseg(0x0);
    dmw0::set_plv0(true);
    dmw0::set_plv3(false);
    dmw0::set_mat(MemoryAccessType::StronglyOrderedUnCached);
    crmd::set_pg(true);
    crmd::set_da(false);

    // 为用户设置页表映射地址翻译模式
    dmw0::set_plv3(true); // 暂未实现页表,故先用内核的映射
}