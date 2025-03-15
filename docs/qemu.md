# 使用qemu-9.1.3的bug修复记录

## 错误描述
经过初步分析后，疑似qemu-9.1.3对loongarch的部分指令存在支持问题：
- 使用gdb调试到vldi指令时，会导致qemu卡住，原因不明

## 解决方法
1. 在`os/.cargo/config.toml`中添加`"-C", "target-feature=-lsx,-lasx"`
2. 使用被注释掉的print宏
3. 使用qemu-9.2.1版本