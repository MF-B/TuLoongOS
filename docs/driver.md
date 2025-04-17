# PCI 驱动说明文档

## 概述
TuLoongOS 使用基于 PCI 总线的硬盘驱动程序来管理存储设备。本文档描述了相关配置和参数信息。

## QEMU 配置参数
启动 QEMU 时，需要添加以下参数来模拟 PCI 设备：

```bash
-device virtio-blk-pci,drive=disk0 \
-drive id=disk0,if=none,format=raw,file=$(FS_IMG)
```

其中 `$(FS_IMG)` 是文件系统映像的路径。

## 重要配置参数

### PCI配置空间基址
```
pci-ecam-base = 0x2000_0000
```

### PCI 总线终止号
```
pci-bus-end = 0x7f
```

### PCI 设备内存范围
```
pci-ranges = [
    [0, 0],
    [0x4000_0000, 0x0002_0000]
]
```
定义了 PCI 设备的内存映射范围。

## 使用说明
在系统启动时，PCI 驱动将自动扫描总线并识别连接的设备，包括上述配置的 virtio-blk-pci 设备。
扫描函数位于`os/src/drivers/pci/pci.rs::init_virtio_blk()`
扫描后自动启用