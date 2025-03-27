use crate::{
    config::{
        PAGE_SIZE, PAGE_SIZE_BITS, USER_STACK_SIZE,
    },
    mm::address::StepByOne,
};
use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use bitflags::*;
use log::{debug, info};


use super::{
    FrameTracker,
    address::{PhysPageNum, VPNRange, VirtAddr, VirtPageNum},
    frame_alloc,
    page_table::{PTEFlags, PageTable},
};

#[derive(Clone, Default)]
pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_perm: MapPermission,
    ) -> Self {
        MapArea {
            vpn_range: VPNRange::new(start_va.floor(), end_va.ceil()),
            data_frames: BTreeMap::new(),
            map_perm,
        }
    }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        let frame = frame_alloc().unwrap();
        ppn = frame.ppn;
        self.data_frames.insert(vpn, frame);
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits().into()).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    #[allow(unused)]
    pub fn ummap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        self.data_frames.remove(&vpn);
        page_table.unmap(vpn);
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }

    #[allow(unused)]
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.ummap_one(page_table, vpn);
        }
    }
    pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }
}


bitflags! {
    #[derive(Default)]
    pub struct MapPermission: usize {
        // const V = 1 << 0;
        // const D = 1 << 1;
        const PLV0 = 1 << 2;
        const PLV1 = 1 << 3;
        // const MAT0 = 1 << 4;
        // const MAT1 = 1 << 5;
        // const G = 1 << 6;
        // const P = 1 << 7;
        const W = 1 << 8;

        const NR = 1 << 61;
        const NX = 1 << 62;
        // const RPLV = 1 << 63;
    }
}

#[derive(Clone, Default)]
pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        MemorySet {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    pub fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);

        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data);
        }

        self.areas.push(map_area);
    }
    /// Assume that no conflicts.
    #[allow(unused)]
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, permission),
            None,
        );
    }

    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();
        // map trampoline
        //memory_set.map_trampoline();
        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::PLV0 | MapPermission::PLV1;
                let ph_flags = ph.flags();
                if !ph_flags.is_read() {
                    map_perm |= MapPermission::NR;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if !ph_flags.is_execute() {
                    map_perm |= MapPermission::NX;
                }
                debug!(
                    "start_vpn: {:x?}, end_vpn: {:x?}",
                    usize::from(start_va) >> PAGE_SIZE_BITS,
                    usize::from(end_va) >> PAGE_SIZE_BITS
                );
                let map_area = MapArea::new(start_va, end_va, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();

                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapPermission::W | MapPermission::PLV0 | MapPermission::PLV1 | MapPermission::NX,
            ),
            None,
        );
        // // map TrapContext
        // memory_set.push(
        //     MapArea::new(
        //         TRAP_CONTEXT.into(),
        //         TRAMPOLINE.into(),
        //         MapType::Framed,
        //         MapPermission::NX | MapPermission::W,
        //     ),
        //     None,
        // );
        info!(
            "user stack bottom: {:#x}, user stack top: {:#x}",
            user_stack_bottom, user_stack_top
        );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub fn token(&self) -> usize {
        // 这里只返回跟页表的地址
        self.page_table.token()
    }
}
