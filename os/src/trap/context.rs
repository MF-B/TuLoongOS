use core::fmt;

use loongArch64::register::{prmd::{self, Prmd}, CpuMode};

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GeneralRegisters {
    pub zero: usize,
    pub ra: usize,
    pub tp: usize,
    pub sp: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub t7: usize,
    pub t8: usize,
    pub u0: usize,
    pub fp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: GeneralRegisters,
    pub era: usize,
    pub prmd: Prmd,
}

impl fmt::Debug for TrapFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrapFrame")
            .field("regs", &self.regs)
            .field("era", &self.era)
            .field("prmd", &format_args!("{:#x}", unsafe { core::mem::transmute::<Prmd, usize>(self.prmd) }))
            .finish()
    }
}

impl TrapFrame {
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut regs= GeneralRegisters::default();
        regs.sp = sp;
        prmd::set_pie(true);
        prmd::set_pplv(CpuMode::Ring3);
        let prmd = prmd::read();
        Self {
            regs,
            era: entry,
            prmd,
        }
    }
}