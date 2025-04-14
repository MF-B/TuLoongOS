use log::info;
use loongArch64::{cpu::*, register::*};


// Task使用的常量
pub const USER_STACK_SIZE: usize = 4096 * 4;
pub const KERNEL_HEAP_SIZE: usize = 0x200000;

// 地址空间
pub const MEMORY_LOW_END: usize = 0x0fffffff;
#[allow(unused)]
pub const MEMORY_HIGH_START: usize = 0x80000000;
#[allow(unused)]
pub const MEMORY_HIGH_END: usize = 0xafffffff;

// TLB使用的常量
pub const PALEN: usize = 48;
pub const VALEN: usize = 48;
pub const PAGE_SIZE: usize = 4096 * 4;
pub const PAGE_SIZE_BITS: usize = 14;
pub const LEVEL_BITS: usize = 11;
pub const LEVELS: usize = 3;



// 打印硬件的相关信息
pub fn print_machine_info() {
    let euen = euen::read();
    info!("基础浮点指令: {:?}", euen.fpe());
    info!("128位向量指令: {:?}", euen.sxe());
    info!("256位向量指令: {:?}", euen.asxe());

    info!("PALEN: {}", get_palen()); //支持的物理地址范围
    info!("VALEN: {}", get_valen()); //支持的虚拟地址范围
    info!("Support MMU-Page :{}", get_mmu_support_page());
    info!("Support Read-only :{}", get_support_read_forbid());
    info!(
        "Support Execution-Protect :{}",
        get_support_execution_protection()
    ); //是否支持执行保护页属性
    info!("Support RPLV: {}", get_support_rplv()); //是否支持rplv页属性
    info!("Support RVA: {}", get_support_rva()); //是否支持虚拟地址缩减
    info!("Support RVAMAX :{}", get_support_rva_len()); //支持的虚拟地址缩减的长度
    info!("Support Page-Size: {:#b}", prcfg2::read().psval()); //支持的页大小,
    match prcfg3::read().tlb_type() {
        0 => {
            info!("No TLB");
        }
        1 => {
            info!("Have MTLB");
        }
        2 => {
            info!("Have STLB + MTLB");
        }
        _ => {
            info!("Unknown TLB");
        }
    }
    info!("MLTB Entry: {}", prcfg3::read().mtlb_entries()); //MTLB的页数量
    info!("SLTB Ways :{}", prcfg3::read().stlb_ways()); //STLB的路数量
    info!("SLTB Entry: {}", prcfg3::read().sltb_sets()); //STLB每一路的项数
    info!("SLTB Page-size: {}", stlbps::read().ps()); //STLB的页大小
    info!("PTE-size: {}", pwcl::read().pte_width()); //PTE的大小
    info!("TLB-RFill entry_point: {:#x}", tlbrentry::read().addr()); //TLB重填的入口地址
    info!("TLB-RFill page-size :{}", tlbrehi::read().ps()); //TLB重填的页大小
    let pwcl = pwcl::read();
    info!(
        "PTE-index-width: {},{}",
        pwcl.ptbase(),
        pwcl.ptwidth()
    ); //PTE的索引宽度
    info!(
        "PGD-index-width: {},{}",
        pwcl.dir1_base(),
        pwcl.dir1_width()
    ); //PGD的索引宽度
    let pwch = pwch::read();
    info!(
        "PMD-index-width: {},{}",
        pwch.dir3_base(),
        pwch.dir3_width()
    ); //PTE的索引宽度
    let crmd = crmd::read();
    info!("DA: {}", crmd.da()); //是否支持DA模式
    info!("PG :{}", crmd.pg()); //是否支持PG模式
    info!("dmwo: {:#x}", dmw0::read().raw()); //映射窗口1
    info!("dmw1: {:#x}", dmw1::read().raw()); //映射窗口2
    info!("PLV: {:?}", crmd.plv()); //当前的特权级
}