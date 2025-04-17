use log::{error, info};
use virtio_drivers::transport::{pci::{bus::{BarInfo, Command, ConfigurationAccess, DeviceFunction, HeaderType, MemoryBarType, PciRoot}, virtio_device_type, PciTransport}, DeviceType};

use crate::drivers::block::{VirtIOBlock, VirtioHal};

use super::{alloc::PciRangeAllocator, dev::{DevError, DevResult}};

// 添加 ConfigurationAccess 实现
const PCI_CONFIG_ADDR_BASE: usize = 0x2000_0000;
struct MyConfigAccess;

impl ConfigurationAccess for MyConfigAccess {
    fn read_word(&self, device_function: virtio_drivers::transport::pci::bus::DeviceFunction, register_offset: u8) -> u32 {
        let bdf = (device_function.bus as u32) << 8
            | (device_function.device as u32) << 3
            | device_function.function as u32;
        let address =
            bdf << 8 | register_offset as u32;
        // Safe because both the `mmio_base` and the address offset are properly aligned, and the
        // resulting pointer is within the MMIO range of the CAM.
        unsafe {
            // Right shift to convert from byte offset to word offset.
            ((PCI_CONFIG_ADDR_BASE as *mut u32).add((address >> 2) as usize)).read_volatile()
        }
    }

    fn write_word(&mut self, device_function: virtio_drivers::transport::pci::bus::DeviceFunction, register_offset: u8, data: u32) {
        let bdf = (device_function.bus as u32) << 8
            | (device_function.device as u32) << 3
            | device_function.function as u32;
        let address =
            bdf << 8 | register_offset as u32;
        // Safe because both the `mmio_base` and the address offset are properly aligned, and the
        // resulting pointer is within the MMIO range of the CAM.
        unsafe {
            // Right shift to convert from byte offset to word offset.
            ((PCI_CONFIG_ADDR_BASE as *mut u32).add((address >> 2) as usize)).write_volatile(data)
        }
    }

    unsafe fn unsafe_clone(&self) -> Self {
        // 简单结构体可以直接复制
        MyConfigAccess
    }
}

/// 初始化 VirtIO 块设备
pub fn init_virtio_blk() -> Option<VirtIOBlock> {
    info!("开始初始化VirtIO块设备...");
    
    // 实例化配置访问对象
    let config_access = MyConfigAccess;
    let mut root = PciRoot::new(config_access);

    info!("扫描PCI总线查找VirtIO设备...");

    // PCI 32-bit MMIO space
    // let mut allocator = Some(PciRangeAllocator::new(0x4000_0000, 0x0002_0000));
    let mut allocator = [(0, 0),(0x4000_0000, 0x0002_0000)]
        .get(1)
        .map(|range| PciRangeAllocator::new(range.0 as u64, range.1 as u64));

    // 扫描有限范围的PCI总线，通常主板上只有少数几个总线
    for bus in 0..=5 {  // 扩大扫描范围，包含0总线
        for (bdf, dev_info) in root.enumerate_bus(bus) {
            info!("PCI {}: {}", bdf, dev_info);
            if dev_info.header_type != HeaderType::Standard {
                continue;
            }
            // 检查是否为VirtIO块设备
            let device_type = virtio_device_type(&dev_info);
            match device_type {
                Some(DeviceType::Block) => {
                    info!("找到VirtIO块设备，开始配置...");
                    // 尝试启用设备
                    if let Ok(_) = config_pci_device(&mut root, bdf, &mut allocator) {
                        info!("VirtIO块设备初始化成功");
                        // 创建 VirtIO 块设备实例
                        match PciTransport::new::<VirtioHal, MyConfigAccess>(&mut root, bdf) {
                            Ok(transport) => return Some(VirtIOBlock::new(transport)),
                            Err(e) => {
                                info!("创建PCI传输层失败: {:?}", e);
                                continue;
                            }
                        }
                    }
                    else {
                        info!("启用VirtIO块设备失败");
                        continue;
                    }
                },
                Some(other) => {
                    info!("跳过非块设备: {:?}", other);
                    continue;
                },
                None => {
                    info!("跳过非VirtIO设备");
                    continue;
                }
            }
        }
    }    
    None
}

const PCI_BAR_NUM: u8 = 6;
fn config_pci_device(
    root: &mut PciRoot<MyConfigAccess>,
    bdf: DeviceFunction,
    allocator: &mut Option<PciRangeAllocator>,
) -> DevResult {
    let mut bar = 0;
    
    info!("开始配置PCI设备: bus={}, device={}, function={}", 
             bdf.bus, bdf.device, bdf.function);

    while bar < PCI_BAR_NUM {
        let info = root.bar_info(bdf, bar).unwrap();
        if let BarInfo::Memory {
            address_type,
            address,
            size,
            ..
        } = info
        {
            // 添加合理性检查
            if size > 0x1000000 { // 超过16MB视为不合理
                bar += 1;
                if info.takes_two_entries() {
                    bar += 1;
                }
                continue;
            }
            // if the BAR address is not assigned, call the allocator and assign it.
            if size > 0 && address == 0 {
                info!("  为BAR{}分配内存, 大小: {:#x}", bar, size);
                let new_addr = match allocator
                    .as_mut()
                    .ok_or(DevError::NoMemory)?
                    .alloc(size as _) {
                        Some(addr) => addr,
                        None => {
                            error!("  内存分配失败: 没有足够的空间");
                            return Err(DevError::NoMemory);
                        }
                    };
                
                info!("  分配地址: {:#x}", new_addr);
                if address_type == MemoryBarType::Width32 {
                    root.set_bar_32(bdf, bar, new_addr as _);
                    info!("  设置32位BAR地址: {:#x}", new_addr);
                } else if address_type == MemoryBarType::Width64 {
                    root.set_bar_64(bdf, bar, new_addr);
                    info!("  设置64位BAR地址: {:#x}", new_addr);
                }
            }
        }

        // read the BAR info again after assignment.
        let info = match root.bar_info(bdf, bar) {
            Ok(info) => info,
            Err(e) => {
                info!("获取BAR{}信息失败: {:?}", bar, e);
                bar += 1;
                continue;
            }
        };
        
        match info {
            BarInfo::IO { address, size } => {
                if address > 0 && size > 0 {
                    info!("  BAR {}: IO  [{:#x}, {:#x})", bar, address, address + size);
                }
            }
            BarInfo::Memory {
                address_type,
                prefetchable,
                address,
                size,
            } => {
                if address > 0 && size > 0 {
                    info!(
                        "  BAR {}: MEM [{:#x}, {:#x}){}{}",
                        bar,
                        address,
                        address + size as u64,
                        if address_type == MemoryBarType::Width64 {
                            " 64bit"
                        } else {
                            ""
                        },
                        if prefetchable { " pref" } else { "" },
                    );
                } else if size > 0 {
                    info!("  BAR {}: 内存大小 {:#x} 但地址为0，配置失败", bar, size);
                }
            }
        }

        bar += 1;
        if info.takes_two_entries() {
            bar += 1;
        }
    }

    // Enable the device.
    let (status, cmd) = root.get_status_command(bdf);
    info!("  设备当前状态: {:#x}, 命令: {:#x}", status, cmd);
    
    let new_cmd = cmd | Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER;
    info!("  设置新命令: {:#x}", new_cmd);
    
    root.set_command(bdf, new_cmd);
    
    // 验证命令是否正确设置
    let (new_status, new_cmd_actual) = root.get_status_command(bdf);
    info!("  命令设置后状态: {:#x}, 命令: {:#x}", new_status, new_cmd_actual);
    
    if new_cmd_actual != new_cmd {
        info!("  命令设置失败，期望 {:#x}，实际 {:#x}", new_cmd, new_cmd_actual);
        return Err(DevError::InvalidParam);
    }
    
    info!("  设备配置成功");
    Ok(())
}